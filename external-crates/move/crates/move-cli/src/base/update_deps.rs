// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;

use move_package_alt::{
    flavor::MoveFlavor,
    package::RootPackage,
    schema::{Environment, EnvironmentName},
};
use move_package_alt_compilation::build_config::BuildConfig;
use move_package_alt_vfs::wrappers::VirtualPath;

/// Re-pin the dependencies of this package.
#[derive(Debug, Clone, Parser)]
pub struct UpdateDeps {
    /// The environment to update dependencies for. If none is provided, all environments'
    /// dependencies will be updated.
    #[arg(name = "environment", short = 'e', long = "environment")]
    environment: Option<EnvironmentName>,
}

impl UpdateDeps {
    pub async fn execute<F: MoveFlavor>(
        &self,
        path: VirtualPath,
        build_config: &BuildConfig,
        env: Environment,
    ) -> anyhow::Result<()> {
        let modes = build_config
            .modes
            .clone()
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let mut root_package = RootPackage::<F>::load_force_repin(path, env, modes).await?;
        root_package.save_lockfile_to_disk()?;
        Ok(())
    }
}
