use anyhow::Result;
use tracing::error;

use std::env;
use std::fs;
use std::process::Command;

fn generate_stagit() -> Result<()> {
    let name = env::current_dir()?
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let repos_dir = format!("{}/repos", env::var("SOFT_SERVE_DATA_PATH")?);
    let dest_dir = format!("{}/web/", env::var("SOFT_SERVE_DATA_PATH")?);
    let dest_repo_dir = format!("{}/{}", dest_dir, name.trim_end_matches(".git"));

    fs::create_dir_all(&dest_repo_dir)?;
    env::set_current_dir(&dest_repo_dir)?;

    Command::new("stagit")
        .arg(format!("{}/{}", repos_dir, name))
        .status()?;

    let stagit_config_src = "/etc/stagit/";
    for entry in fs::read_dir(stagit_config_src)? {
        let file = entry?;
        fs::copy(
            file.path(),
            dest_repo_dir.clone() + "/" + file.file_name().to_string_lossy().as_ref(),
        )?;
    }

    let mut stagit_index_args = vec![];
    for entry in fs::read_dir(&repos_dir)? {
        let repo = entry?;
        stagit_index_args.push(repo.path().to_string_lossy().to_string() + "/");
    }

    let output = Command::new("stagit-index")
        .args(stagit_index_args)
        .output()?;

    if !output.status.success() {
        error!("failed to generate web index");
    } else {
        fs::write(format!("{}/index.html", dest_dir), output.stdout)?;
    }

    for entry in fs::read_dir(stagit_config_src)? {
        let file = entry?;
        fs::copy(
            file.path(),
            format!("{}/{}", dest_dir, file.file_name().to_string_lossy()),
        )?;
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    generate_stagit()?;

    Ok(())
}
