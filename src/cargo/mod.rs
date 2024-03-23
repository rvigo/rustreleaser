use crate::{arch_os_matrix::ArchOsMatrixEntry, build::Build};
use anyhow::{bail, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;
use tokio_stream::StreamExt;

const DEFAULT_CARGO_FILE_NAME: &str = "Cargo.toml";
const DEFAULT_CARGO_BIN_NAME: &str = "cargo";
const DEFAULT_RUSTUP_BIN_NAME: &str = "rustup";

// TODO add multi target
pub async fn build(build: &Build) -> Result<()> {
    check_cargo()?;
    check_cargo_project()?;
    if build.is_multi_target() {
        build_multi(Vec::from(build.to_owned())).await?;
    } else {
        build_single().await?;
    };
    Ok(())
}

pub async fn build_single() -> Result<()> {
    Command::new(DEFAULT_CARGO_BIN_NAME)
        .args(["build", "--release"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn build_multi(matrix: Vec<ArchOsMatrixEntry>) -> Result<Vec<CustomCommand>> {
    for entry in &matrix {
        check_cargo_target(entry).await?;
    }

    let commands = create_commands(matrix).await?;

    let mut stream = tokio_stream::iter(commands);

    let mut s = vec![];
    while let Some(mut c) = stream.next().await {
        let exit_status = c.command.spawn()?.wait().await?;
        if exit_status.success() {
            log::info!("Build successful for {}", c.target.to_string());
            c.set_success();
        } else {
            log::error!("Build failed for {}", c.target.to_string());
        }

        s.push(c)
    }
    Ok(s)
}

async fn create_commands(matrix: Vec<ArchOsMatrixEntry>) -> Result<Vec<CustomCommand>> {
    let commands = matrix.iter().map(|entry| {
        log::info!("creating build command for {}", entry.to_string());

        let mut command = Command::new(DEFAULT_CARGO_BIN_NAME);
        command
            .args(["build", "-q", "--release", "--target", &entry.to_string()])
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit());

        let custom_command = CustomCommand::new(command, entry.to_owned());
        custom_command
    });

    Ok(commands.collect())
}

fn check_cargo() -> Result<()> {
    match which::which(DEFAULT_CARGO_BIN_NAME) {
        Ok(_) => Ok(()),
        Err(error) => {
            log::error!("Some strange error occurred :(");
            bail!(anyhow::anyhow!(
                "Some strange error occurred :( {:?}",
                error
            ));
        }
    }
}

async fn check_cargo_target(target: &ArchOsMatrixEntry) -> Result<()> {
    let mut command = Command::new(DEFAULT_RUSTUP_BIN_NAME);
    command.args(["rustup", "target", "add", &target.to_string()]);
    command.spawn()?.wait().await?;
    Ok(())
}

fn check_cargo_project() -> Result<PathBuf> {
    let mut path = PathBuf::from(".");
    let file = Path::new(DEFAULT_CARGO_FILE_NAME);

    loop {
        path.push(file);

        if path.is_file() {
            log::info!("Found Cargo.toml at: {}", path.display());
            return Ok(path);
        }

        if !(path.pop() && path.pop()) {
            // remove file && remove parent
            panic!("Cargo.toml not found in the current directory or any of its parents!");
        }
    }
}

pub struct CustomCommand {
    command: Command,
    target: ArchOsMatrixEntry,
    success: bool,
}

impl CustomCommand {
    fn new(command: Command, target: ArchOsMatrixEntry) -> Self {
        Self {
            command,
            target,
            success: false,
        }
    }

    fn set_success(&mut self) {
        self.success = true;
    }
}
