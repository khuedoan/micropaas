use anyhow::{Result, anyhow};
use clap::Parser;
use serde::Serialize;
use std::{
    env, fs,
    process::{Command, Stdio},
};
use tracing::{info, warn};

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
        > 1
    // TODO ??
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

fn deploy(image: &Image, gitops_repo: &str, repository: &str) -> Result<String> {
    info!("Deploying {image:?}");
    let default_branch = env::var("DEFAULT_BRANCH").unwrap_or("master".to_string());
    let gitops_bare_dir = format!("/var/lib/micropaas/repos/{gitops_repo}.git");
    env::set_current_dir(&gitops_bare_dir)?;

    info!("Setting up worktree for {default_branch} from {gitops_bare_dir}");
    let worktree_dir = format!("{}/{}", gitops_bare_dir, default_branch);
    Command::new("git")
        .args(&[
            "worktree",
            "add",
            "--quiet",
            &worktree_dir,
            &default_branch,
        ])
        .status()?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow!("failed to create new worktree"))?;


    env::set_current_dir(&worktree_dir)?;

    let app_values_file = format!("apps/{repository}/values.yaml");

    info!("Updating image tag in {app_values_file}");
    let content = std::fs::read_to_string(&app_values_file)?;
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let Some(tag) = yaml
        .get_mut("app-template")
        .and_then(|v| v.get_mut("controllers"))
        .and_then(|v| v.get_mut("main"))
        .and_then(|v| v.get_mut("containers"))
        .and_then(|v| v.get_mut("main"))
        .and_then(|v| v.get_mut("image"))
        .and_then(|v| v.get_mut("tag"))
    {
        *tag = serde_yaml::Value::String(image.tag.clone());
    }

    let new_yaml = serde_yaml::to_string(&yaml)?;

    Command::new("git")
        .args(&[
            "diff"
        ])
        .env_remove("GIT_DIR")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow!("failed to run diff"))?;

    info!("Committing changes");
    std::fs::write(app_values_file, new_yaml)?;
    Command::new("git")
        .args(&[
            "add",
            "."
        ])
        .env_remove("GIT_DIR")
        .status()?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow!("failed to stage change"))?;

    Command::new("git")
        .args(&[
            "-c",
            &format!(
                "user.name={}",
                env::var("GIT_USER_NAME").unwrap_or("Bot".to_string())
            ),
            "-c",
            &format!(
                "user.email={}",
                env::var("GIT_USER_EMAIL").unwrap_or("bot@example.com".to_string())
            ),
            "commit",
            "--message",
            &format!("chore({}): update image tag to {}", repository, image.tag),
        ])
        .env_remove("GIT_DIR")
        .status()?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow!("failed to commit change"))?;

    env::set_current_dir(&gitops_bare_dir)?;
    Command::new("git")
        .args(&["worktree", "remove", "--force", &default_branch])
        .output()?;

    Ok(repository.to_string())
}

fn trigger_sync(repository: &str) -> Result<()> {
    #[derive(Debug, Serialize)]
    struct Commit {
        added: Vec<String>,
        modified: Vec<String>,
        removed: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    struct Repository {
        html_url: String,
        default_branch: String,
    }

    #[derive(Debug, Serialize)]
    struct WebhookPayload {
        r#ref: String,
        before: String,
        after: String,
        commits: Vec<Commit>,
        repository: Repository,
    }

    let argocd_webhook_endpoint = env::var("ARGOCD_WEBHOOK_ENDPOINT")
        .unwrap_or("http://argocd-server.argocd.svc.cluster.local/api/webhook".to_string());
    // TODO https://github.com/argoproj/argo-cd/issues/12268
    // Pretending to be GitHub for now, read this code to understand the required payload
    // https://github.com/argoproj/argo-cd/blob/master/util/webhook/webhook.go
    reqwest::blocking::Client::new()
        .post(&argocd_webhook_endpoint)
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", "push")
        .json(&WebhookPayload {
            r#ref: "refs/heads/master".to_string(),
            before: "0000000000000000000000000000000000000000".to_string(),
            after: "0000000000000000000000000000000000000000".to_string(),
            commits: vec![Commit {
                added: vec![],
                modified: vec![],
                removed: vec![],
            }],
            repository: Repository {
                html_url: format!("http://micropaas.micropaas.svc.cluster.local:8080/{repository}"),
                default_branch: "master".to_string(),
            },
        })
        .send()?;

    Ok(())
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
                .and_then(|remote_image| {
                    let gitops_repo = env::var("GITOPS_REPO").unwrap_or("gitops".to_string());
                    deploy(&remote_image, &gitops_repo, &repository)
                })
                .and_then(|app| trigger_sync(app.as_ref()))?;
        } else {
            warn!("No REGISTRY_HOST set, skipping push and deploy");
        }
    }

    Ok(())
}
