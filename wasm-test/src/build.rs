use move_package::BuildConfig as MoveBuildConfig;
use move_package::source_package::parsed_manifest::{Dependencies, Dependency, DependencyKind, GitInfo, InternalDependency, PackageName};
use std::path::{Path, PathBuf};
use sui_move_build::{BuildConfig, SuiPackageHooks};

wit_bindgen::generate!({
    world: "build",
});
export!(Build);

pub struct Build;

fn build_package(path: &Path, install_dir: PathBuf, system_packages_path: &Path) {
    new_build_config(install_dir, system_packages_path).build(path).unwrap();
}

impl Guest for Build {
    fn build_package(path: String, install_dir: String, system_packages_path: String) {
        println!("-1");
        build_package(
            Path::new(&path),
            Path::new(&install_dir).to_path_buf(),
            Path::new(&system_packages_path),
        );
    }
}

fn new_build_config(install_dir: PathBuf, system_packages_path: &Path) -> BuildConfig {
    move_package::package_hooks::register_package_hooks(Box::new(SuiPackageHooks));

    let config = MoveBuildConfig {
        default_flavor: Some(move_compiler::editions::Flavor::Sui),

        lock_file: Some(install_dir.join("Move.lock")),
        install_dir: Some(install_dir),
        silence_warnings: true,
        lint_flag: move_package::LintFlag::LEVEL_NONE,
        implicit_dependencies: implicit_deps(system_packages_path),
        ..MoveBuildConfig::default()
    };
    BuildConfig {
        config,
        run_bytecode_verifier: true,
        print_diags_to_stderr: false,
        chain_id: None,
    }
}

fn implicit_deps(system_packages_path: &Path) -> Dependencies {
    let mut results = Vec::new();

    let system_packages = [
        ("MoveStdlib", "move-stdlib"),
        ("Sui", "sui-framework")
    ];

    for (package_name, package_folder) in system_packages {
        let dependency_path = system_packages_path.join(package_folder);
        println!("system_packages_path: {system_packages_path:?}");
        println!("package_folder: {package_folder:?}");
        println!("dependency_path: {dependency_path:?}");

        let dependency = Dependency::Internal(InternalDependency {
            kind: DependencyKind::Local(dependency_path),
            subst: None,
            digest: None,
            dep_override: true,
        });

        results.push((PackageName::from(package_name), dependency));
    }

    results.into_iter().collect()
}
