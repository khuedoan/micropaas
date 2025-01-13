use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json::json;
use std::{
    env, fmt, fs,
    process::{Command, Stdio},
};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

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

impl Image {
    fn push(self) -> Result<Self> {
        info!("pushing {self}");
        Command::new("docker")
            .args(&["push", "--quiet", &format!("{self}")])
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("failed to push image {self}"))?;

        Ok(self)
    }

    #[tracing::instrument(level = "debug")]
    fn deploy(self, gitops_repo: &str, repository: &str) -> Result<()> {
        info!("deploying {self} to via {gitops_repo}");
        let default_branch = env::var("DEFAULT_BRANCH").unwrap_or("master".to_string());
        let gitops_bare_dir = format!("/var/lib/micropaas/repos/{gitops_repo}.git");
        env::set_current_dir(&gitops_bare_dir)?;

        debug!("setting up worktree for {default_branch} from {gitops_bare_dir}");
        let worktree_dir = format!("{}/{}", gitops_bare_dir, default_branch);
        Command::new("git")
            .args(&["worktree", "add", "--quiet", &worktree_dir, &default_branch])
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("failed to create new worktree"))?;

        debug!("worktree dir: {worktree_dir}");
        env::set_current_dir(&worktree_dir)?;

        let app_values_file = format!("apps/{repository}/values.yaml");
        debug!("updating image tag in {app_values_file}");
        let content = std::fs::read_to_string(&app_values_file)?;
        debug!("app values file content: {content}");
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
            *tag = serde_yaml::Value::String(self.tag.clone());
        }

        let new_yaml = serde_yaml::to_string(&yaml)?;
        debug!("new app values file content: {new_yaml}");

        std::fs::write(app_values_file, new_yaml)?;

        if Command::new("git")
            .args(&["diff", "--color=always", "--unified=0", "--exit-code"])
            .env_remove("GIT_DIR")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
            .success()
        {
            warn!("no changes to commit");
        } else {
            debug!("committing changes");
            Command::new("git")
                .args(&["add", "."])
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
                    &format!("chore({}): update image tag to {}", repository, self.tag),
                ])
                .env_remove("GIT_DIR")
                .status()?
                .success()
                .then_some(())
                .ok_or_else(|| anyhow!("failed to commit change"))?;
        }

        env::set_current_dir(&gitops_bare_dir)?;
        Command::new("git")
            .args(&["worktree", "remove", "--force", &default_branch])
            .output()?;

        Ok(())
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}:{}", self.registry, self.repository, self.tag)
    }
}

#[tracing::instrument(level = "debug")]
fn setup_workspace(new_object: &str) -> Result<()> {
    let workspace_dir =
        std::str::from_utf8(&Command::new("mktemp").args(&["-d"]).output()?.stdout)?
            .trim()
            .to_string();
    debug!("workspace dir: {}", workspace_dir);

    Command::new("git")
        .args(&["worktree", "add", "--quiet", &workspace_dir, new_object])
        .output()?;

    env::set_current_dir(&workspace_dir)?;

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn ci(repository: &str, ref_name: &str, old_object: &str, new_object: &str) -> Result<()> {
    if fs::metadata("flake.nix").is_ok()
        && fs::metadata("flake.lock").is_ok()
        && fs::metadata("Makefile").is_ok()
        && fs::read_to_string("Makefile")
            .map(|contents| contents.lines().any(|line| line == "ci:"))
            .unwrap_or(false)
    {
        info!("running CI (this may take a while to download dependencies)");

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
                &format!("CACHE_DIR=/var/cache/micropaas/{repository}"),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
    }

    Ok(())
}

#[derive(Debug)]
enum Builder {
    Dockerfile,
    Nixpacks,
}

impl Builder {
    #[tracing::instrument(level = "debug")]
    fn detect() -> Result<Option<Self>> {
        if fs::metadata("Dockerfile").is_ok() {
            Ok(Some(Builder::Dockerfile))
        } else if Command::new("nixpacks")
            .args(&["detect", "."])
            .output()?
            .stdout
            .trim_ascii()
            .len()
            > 1
        {
            Ok(Some(Builder::Nixpacks))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(level = "debug")]
    fn build(&self, registry: &str, repository: &str, new_object: &str) -> Result<Image> {
        let image = Image {
            registry: registry.to_string(),
            repository: repository.to_string(),
            tag: new_object.to_string(),
        };
        match self {
            Builder::Dockerfile => {
                info!("building container image with Dockerfile");
                Command::new("docker")
                    .args(&["build", ".", "--tag", &format!("{image}")])
                    .output()?;
                Ok(image)
            }
            Builder::Nixpacks => {
                info!("building container image with Nixpacks");
                Command::new("nixpacks")
                    .args(&[
                        "build",
                        ".",
                        "--cache-key",
                        &repository,
                        "--tag",
                        &format!("{image}"),
                    ])
                    .env("CLICOLOR_FORCE", "true")
                    .stdout(Stdio::inherit())
                    .output()?;
                Ok(image)
            }
        }
    }
}

#[tracing::instrument(level = "debug")]
fn trigger_sync(webhook_endpoint: &str, repository: &str) -> Result<()> {
    debug!("triggering sync for {repository}");
    // TODO https://github.com/argoproj/argo-cd/issues/12268
    // Pretending to be GitHub for now, read this code to understand the required payload
    // https://github.com/argoproj/argo-cd/blob/master/util/webhook/webhook.go
    let payload = json!({
        "ref": "refs/heads/master",
        "before": "0000000000000000000000000000000000000000",
        "after": "0000000000000000000000000000000000000000",
        "commits": [
            {
                "added": [],
                "modified": [],
                "removed": []
            }
        ],
        "repository": {
            "html_url": format!("http://micropaas.micropaas.svc.cluster.local:8080/{repository}"),
            "default_branch": "master"
        }
    });
    debug!("webhook payload: {:?}", payload);

    let response = reqwest::blocking::Client::new()
        .post(webhook_endpoint)
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", "push")
        .json(&payload)
        .send()?;
    debug!("webhook response: {:?}", response);

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .without_time()
        .init();

    let args = Args::parse();

    // Configuration
    let registry = env::var("REGISTRY_HOST")?;
    let webhook_endpoint = env::var("ARGOCD_WEBHOOK_ENDPOINT")?;
    let gitops_repo = env::var("GITOPS_REPO")?;

    // Runtime
    let repository = env::var("SOFT_SERVE_REPO_NAME")?;

    setup_workspace(&args.new_object)?;

    ci(
        &repository,
        &args.ref_name,
        &args.old_object,
        &args.new_object,
    )?;

    if let Some(builder) = Builder::detect()? {
        builder
            .build(&registry, &repository, &args.new_object)?
            .push()?
            .deploy(&gitops_repo, &repository)?;
        trigger_sync(&webhook_endpoint, &gitops_repo)?;
    } else {
        info!("no buildable code detected");
    }

    Ok(())
}
