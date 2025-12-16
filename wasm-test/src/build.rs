use anyhow::Error;
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::codes::{
    DiagnosticInfo as MoveDiagnosticInfo, Severity as MoveSeverity,
};
use move_compiler::diagnostics::{
    Diagnostic as MoveDiagnostic, Diagnostics as MoveDiagnostics,
    DiagnosticsFormat as MoveDiagnosticsFormat, report_diagnostics_to_buffer,
};
use move_core_types::account_address::AccountAddress;
use move_ir_types::location::Loc;
use move_package::BuildConfig as MoveBuildConfig;
use move_package::compilation::build_plan::BuildPlan;
use move_package::compilation::compiled_package::CompiledPackage as MoveCompiledPackage;
use move_package::resolution::resolution_graph::ResolvedGraph;
use move_package::source_package::parsed_manifest::NamedAddress;
use move_package::source_package::parsed_manifest::{
    Dependencies, Dependency, DependencyKind, InternalDependency, PackageName,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use sui_move_build::{
    BuildConfig, CompiledPackage, SuiPackageHooks, collect_bytecode_deps, decorate_warnings,
    gather_published_ids, verify_bytecode,
};
use sui_types::error::{SuiErrorKind, SuiResult};
use sui_types::move_package::FnInfoMap;

wit_bindgen::generate!({
    world: "build",
});
export!(Build);

pub struct Build;

impl Guest for Build {
    fn build_package(
        path: String,
        install_dir: String,
        system_packages_path: String,
    ) -> Result<BuildResult, BuildError> {
        let path = Path::new(&path);
        let install_dir = Path::new(&install_dir).to_path_buf();
        let system_packages_path = Path::new(&system_packages_path);

        build_package(&path, install_dir, system_packages_path).map_err(|(error, diagnostics)| {
            BuildError {
                message: error.to_string(),
                diagnostics: convert_diagnostics(diagnostics),
            }
        })
    }
}

fn build_package(
    path: &Path,
    install_dir: PathBuf,
    system_packages_path: &Path,
) -> Result<BuildResult, (anyhow::Error, MoveDiagnostics)> {
    let build_config = new_build_config(install_dir, system_packages_path);

    let chain_id = build_config.chain_id.clone();
    let resolution_graph = build_config
        .resolution_graph(path, chain_id.clone())
        .map_err(|e| (Error::from(e), MoveDiagnostics::new()))?;

    let (compiled, warning_diagnostics) =
        build_from_resolution_graph(resolution_graph, true, chain_id)?;

    let dependency_ids = compiled
        .get_published_dependencies_ids()
        .into_iter()
        .map(|id| id.into_bytes().to_vec())
        .collect();
    let compiled_modules = compiled.get_package_bytes(false);

    let result = BuildResult {
        dependency_ids,
        compiled_modules,
        warning_diagnostics: convert_diagnostics(warning_diagnostics),
    };

    Ok(result)
}

fn build_from_resolution_graph(
    mut resolution_graph: ResolvedGraph,
    run_bytecode_verifier: bool,
    chain_id: Option<String>,
) -> Result<(CompiledPackage, MoveDiagnostics), (anyhow::Error, MoveDiagnostics)> {
    let (published_at, dependency_ids) = gather_published_ids(&resolution_graph, chain_id);

    // Ensure the compiler substitutes published dependency addresses into bytecode.
    for (name, id) in &dependency_ids.published {
        resolution_graph
            .build_options
            .additional_named_addresses
            .insert(name.to_string(), AccountAddress::from(*id));

        if let Some(pkg) = resolution_graph.package_table.get_mut(name) {
            pkg.resolved_table
                .insert(NamedAddress::from(name.as_str()), AccountAddress::from(*id));
        }

        // Also update the root package's bindings so it links to published deps at their on-chain addresses.
        let root = resolution_graph.root_package();
        if let Some(root_pkg) = resolution_graph.package_table.get_mut(&root) {
            root_pkg
                .resolved_table
                .insert(NamedAddress::from(name.as_str()), AccountAddress::from(*id));
        }
    }

    let bytecode_deps = collect_bytecode_deps(&resolution_graph).unwrap();

    // compile!
    let (package, fn_info, warning_diagnostics) = compile_package(&resolution_graph)?;

    if run_bytecode_verifier {
        verify_bytecode(&package, &fn_info).unwrap();
    }

    let compiled_package = CompiledPackage {
        package,
        published_at,
        dependency_ids,
        bytecode_deps,
        dependency_graph: resolution_graph.graph,
    };

    Ok((compiled_package, warning_diagnostics))
}

fn compile_package(
    resolution_graph: &ResolvedGraph,
) -> Result<(MoveCompiledPackage, FnInfoMap, MoveDiagnostics), (anyhow::Error, MoveDiagnostics)> {
    let build_plan = BuildPlan::create(resolution_graph).unwrap();
    let mut diagnostics = MoveDiagnostics::new();

    let mut fn_info = None;
    let compiled_pkg_result = build_plan.compile_with_driver(&mut std::io::sink(), |compiler| {
        let (files, units_res) = compiler.build()?;
        match units_res {
            Ok((units, warning_diags)) => {
                decorate_warnings(warning_diags.clone(), Some(&files));
                fn_info = Some(BuildConfig::fn_info(&units));
                diagnostics = warning_diags;
                Ok((files, units))
            }
            Err(error_diags) => {
                // with errors present don't even try decorating warnings output to avoid
                // clutter
                diagnostics = error_diags.clone();
                assert!(!error_diags.is_empty());
                let diags_buf =
                    report_diagnostics_to_buffer(&files, error_diags, /* color */ true);
                if let Err(err) = std::io::stderr().write_all(&diags_buf) {
                    anyhow::bail!("Cannot output compiler diagnostics: {}", err);
                }
                anyhow::bail!("Compilation error");
            }
        }
    });

    match compiled_pkg_result {
        Ok(compiled_pkg) => Ok((compiled_pkg, fn_info.unwrap(), diagnostics)),
        Err(err) => Err((err, diagnostics)),
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

    let system_packages = [("MoveStdlib", "move-stdlib"), ("Sui", "sui-framework")];

    for (package_name, package_folder) in system_packages {
        let dependency_path = system_packages_path.join(package_folder);

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

fn convert_diagnostics(diagnostics: MoveDiagnostics) -> Diagnostics {
    let format = convert_format(diagnostics.format);
    match diagnostics.diags {
        Some(inner) => Diagnostics {
            format,
            diagnostics: inner
                .diagnostics
                .into_iter()
                .map(convert_diagnostic)
                .collect(),
            filtered_source_diagnostics: inner
                .filtered_source_diagnostics
                .into_iter()
                .map(convert_diagnostic)
                .collect(),
        },
        None => Diagnostics {
            format,
            diagnostics: vec![],
            filtered_source_diagnostics: vec![],
        },
    }
}

fn convert_diagnostic(diag: MoveDiagnostic) -> Diagnostic {
    let MoveDiagnostic {
        info,
        primary_label,
        secondary_labels,
        notes,
    } = diag;

    Diagnostic {
        info: convert_info(info),
        primary_label: convert_label(primary_label),
        secondary_labels: secondary_labels.into_iter().map(convert_label).collect(),
        notes,
    }
}

fn convert_info(info: MoveDiagnosticInfo) -> DiagnosticInfo {
    DiagnosticInfo {
        severity: convert_severity(info.severity()),
        category: info.category(),
        code: info.code(),
        external_prefix: info.external_prefix().map(|s| s.to_string()),
        message: info.message().to_string(),
    }
}

fn convert_label((loc, message): (Loc, String)) -> DiagnosticLabel {
    DiagnosticLabel {
        location: convert_loc(loc),
        message,
    }
}

fn convert_loc(loc: Loc) -> Location {
    let file_hash: FileHash = loc.file_hash();
    Location {
        file_hash: file_hash.0.to_vec(),
        start: loc.start(),
        end: loc.end(),
    }
}

fn convert_severity(severity: MoveSeverity) -> Severity {
    match severity {
        MoveSeverity::Note => Severity::Note,
        MoveSeverity::Warning => Severity::Warning,
        MoveSeverity::NonblockingError => Severity::NonblockingError,
        MoveSeverity::BlockingError => Severity::BlockingError,
        MoveSeverity::Bug => Severity::Bug,
    }
}

fn convert_format(format: MoveDiagnosticsFormat) -> DiagnosticsFormat {
    match format {
        MoveDiagnosticsFormat::Text => DiagnosticsFormat::Text,
        MoveDiagnosticsFormat::JSON => DiagnosticsFormat::Json,
    }
}
