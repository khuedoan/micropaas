use clap::Parser;
use std::{
    env,
    fs,
    path::Path,
    process::Command,
};
use tracing::info;
use anyhow::Result;

#[derive(Debug, Parser)]
struct Args {
    ref_name: String,
    old_object: String,
    new_object: String,
}

#[derive(Debug)]
struct Image {
    registry: String,
    repository: String,
    tag: String,
}

fn setup_workspace(new_object: &String) -> Result<()> {
    let mktemp = Command::new("mktemp")
        .args(&["-d"])
        .output()?;

    let workspace_dir = std::str::from_utf8(&mktemp.stdout)?
        .trim()
        .to_string();

    Command::new("git")
        .args(&[
            "worktree",
            "add",
            "--quiet",
            &workspace_dir,
            new_object,
        ])
        .output()?;
    
    env::set_current_dir(&workspace_dir)?;

    Ok(())
}

fn ci(ref_name: &String, old_object: &String, new_object: &String) -> Result<()> {
    info!("Workspace directory: {:?}", env::current_dir().unwrap());
    if fs::metadata("flake.nix").is_ok()
        && fs::metadata("flake.lock").is_ok()
        && fs::metadata("Makefile").is_ok()
        && fs::read_to_string("Makefile")
            .map(|contents| contents.lines().any(|line| line == "ci:"))
            .unwrap_or(false)
    {
        info!("running CI {ref_name} {old_object} {new_object}");
    }
    Err(anyhow::anyhow!("not implemented"))
}

fn build() -> Result<Image> {
    info!("building");
    Err(anyhow::anyhow!("not implemented"))
}

fn push(image: &Image) -> Result<()> {
    info!("pushing {image:?}");
    Err(anyhow::anyhow!("not implemented"))
}

fn deploy(image: &Image) -> Result<()> {
    info!("deploying {image:?}");
    Err(anyhow::anyhow!("not implemented"))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let args = Args::parse();

    setup_workspace(&args.new_object)?;
    ci(&args.ref_name, &args.old_object, &args.new_object)?;

    if let Ok(image) = build() {
        push(&image)?;
        deploy(&image)?;
    }

    Ok(())
}
