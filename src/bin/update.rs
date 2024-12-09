use clap::Parser;
use std::fs;
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

fn ci() -> Result<(), String> {
    info!("running CI");
    Ok(())
}

fn build() -> Result<Image, String> {
    info!("build");
    Ok(Image {
        registry: "docker.io".to_string(),
        repository: "khuedoan/blog".to_string(),
        tag: "latest".to_string(),
    })
}

fn push(image: &Image) -> Result<(), String> {
    info!("pushing {image:?}");
    Ok(())
}

fn deploy(image: &Image) -> Result<(), String> {
    info!("deploying {image:?}");
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let args = Args::parse();

    info!("{:?}", args);

    if fs::metadata("flake.nix").is_ok()
        && fs::metadata("flake.lock").is_ok()
        && fs::metadata("Makefile").is_ok()
        && fs::read_to_string("Makefile")
            .map(|contents| contents.lines().any(|line| line == "ci:"))
            .unwrap_or(false)
    {
        ci().unwrap();
    }

    if let Ok(image) = build() {
        push(&image).unwrap();
        deploy(&image).unwrap();
    }
}
