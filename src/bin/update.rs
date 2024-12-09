struct Image {
    registry: String,
    repository: String,
    tag: String,
}

fn build() -> Result<Image, String> {
    Ok(Image {
        registry: "registry".to_string(),
        repository: "repository".to_string(),
        tag: "tag".to_string(),
    })
}

fn main() {
    println!("update hook");
}
