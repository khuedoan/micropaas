use anyhow::Result;
use clap::Parser;
use std::{
    env, fs,
    process::{Command, Stdio},
};
use tracing::info;

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
    let workspace_dir =
        std::str::from_utf8(&Command::new("mktemp").args(&["-d"]).output()?.stdout)?
            .trim()
            .to_string();

    Command::new("git")
        .args(&["worktree", "add", "--quiet", &workspace_dir, new_object])
        .output()?;

    env::set_current_dir(&workspace_dir)?;

    Ok(())
}

fn ci(ref_name: &String, old_object: &String, new_object: &String) -> Result<()> {
    if fs::metadata("flake.nix").is_ok()
        && fs::metadata("flake.lock").is_ok()
        && fs::metadata("Makefile").is_ok()
        && fs::read_to_string("Makefile")
            .map(|contents| contents.lines().any(|line| line == "ci:"))
            .unwrap_or(false)
    {
        info!("Running CI (this may take a while to donwload dependencies)");

        Command::new("nix")
            .args(&[
                "develop",
                "--quiet",
                "--command",
                "make",
                "ci",
                &format!("REF_NAME={ref_name}"),
                &format!("OLD_OBJECT={old_object}"),
                &format!("NEW_OBJECT={new_object}"),
                &format!("CACHE_DIR=/tmp"),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
    }

    Ok(())
}

enum BuildType {
    Dockerfile,
    Nixpacks,
}

fn build_type() -> Result<Option<BuildType>> {
    if fs::metadata("Dockerfile").is_ok() {
        Ok(Some(BuildType::Dockerfile))
    } else if Command::new("nixpacks")
        .args(&["detect", "."])
        .output()?
        .stdout
        .len()
        > 0
    {
        Ok(Some(BuildType::Nixpacks))
    } else {
        Ok(None)
    }
}

fn build(new_object: &String) -> Result<Option<Image>> {
    let repository = env::var("SOFT_SERVE_REPO_NAME")?;
    let tag = new_object;
    match build_type()? {
        Some(BuildType::Dockerfile) => {
            info!("Building Dockerfile");
            Command::new("docker")
                .args(&["build", ".", "--tag", &format!("{repository}:{tag}")])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
            Ok(Some(Image {
                registry: "localhost".to_string(),
                repository,
                tag: tag.to_string(),
            }))
        }
        Some(BuildType::Nixpacks) => {
            info!("Building with Nixpacks");
            Command::new("nixpacks")
                .args(&[
                    "build",
                    ".",
                    "--cache-key",
                    &repository,
                    "--tag",
                    &format!("{repository}:{tag}"),
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
            Ok(Some(Image {
                registry: "localhost".to_string(),
                repository,
                tag: tag.to_string(),
            }))
        }
        None => Ok(None),
    }
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

    if let Ok(Some(image)) = build(&args.new_object) {
        push(&image)?;
        deploy(&image)?;
    }

    Ok(())
}
