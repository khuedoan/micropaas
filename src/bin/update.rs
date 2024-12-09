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

fn setup_workspace(new_object: &str) -> Result<()> {
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

fn ci(repository: &str, ref_name: &str, old_object: &str, new_object: &str) -> Result<()> {
    if fs::metadata("flake.nix").is_ok()
        && fs::metadata("flake.lock").is_ok()
        && fs::metadata("Makefile").is_ok()
        && fs::read_to_string("Makefile")
            .map(|contents| contents.lines().any(|line| line == "ci:"))
            .unwrap_or(false)
    {
        info!("Running CI (this may take a while to download dependencies)");

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
                &format!("CACHE_DIR=/var/cache/micropaas/{repository}/{ref_name}"),
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

fn build(repository: &str, new_object: &str) -> Result<Option<Image>> {
    let tag = new_object;
    match build_type()? {
        Some(BuildType::Dockerfile) => {
            info!("Building Dockerfile");
            Command::new("docker")
                .args(&[
                    "build",
                    ".",
                    "--tag",
                    &format!("localhost/{repository}:{tag}"),
                ])
                .output()?;
            Ok(Some(Image {
                registry: "localhost".to_string(),
                repository: repository.to_string(),
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
                    &format!("localhost/{repository}:{tag}"),
                ])
                .output()?;
            Ok(Some(Image {
                registry: "localhost".to_string(),
                repository: repository.to_string(),
                tag: tag.to_string(),
            }))
        }
        None => Ok(None),
    }
}

fn push(registry: &str, image: &Image) -> Result<Image> {
    let remote_image = Image {
        registry: registry.to_string(),
        repository: image.repository.clone(),
        tag: image.tag.clone(),
    };

    Command::new("docker")
        .args(&[
            "tag",
            &format!("{}/{}:{}", image.registry, image.repository, image.tag),
            &format!(
                "{}/{}:{}",
                remote_image.registry, remote_image.repository, remote_image.tag
            ),
        ])
        .output()?;

    info!("Pushing {remote_image:?}");
    Command::new("docker")
        .args(&[
            "push",
            "--quiet",
            &format!(
                "{}/{}:{}",
                remote_image.registry, remote_image.repository, remote_image.tag
            ),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(remote_image)
}

fn deploy(image: &Image) -> Result<String> {
    info!("deploying {image:?}");
    Err(anyhow::anyhow!("not implemented"))
}

fn trigger_sync(_app: &str) -> Result<()> {
    info!("triggering sync");
    Err(anyhow::anyhow!("not implemented"))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let args = Args::parse();
    let repository = env::var("SOFT_SERVE_REPO_NAME")?;

    setup_workspace(&args.new_object)?;

    ci(
        &repository,
        &args.ref_name,
        &args.old_object,
        &args.new_object,
    )?;

    if let Ok(Some(image)) = build(&repository, &args.new_object) {
        if let Ok(registry) = env::var("REGISTRY_HOST") {
            push(&registry, &image)
                .and_then(|remote_image| deploy(&remote_image))
                .and_then(|app| trigger_sync(app.as_ref()))?;
        } else {
            info!("No REGISTRY_HOST set, skipping push and deploy");
        }
    }

    Ok(())
}
