use std::{fs, process::Command};

pub fn setup() {
    Command::new("docker")
        .arg("compose")
        .arg("up")
        .arg("--detach")
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("failed to start Docker Compose");

    let testdata_dir = "tests/testdata";
    fs::create_dir_all(testdata_dir).unwrap();

    Command::new("git")
        .arg("clone")
        .arg("https://github.com/khuedoan/horus")
        .arg(format!("{testdata_dir}/horus"))
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();

    Command::new("git")
        .arg("clone")
        .arg("https://github.com/khuedoan/blog")
        .arg(format!("{testdata_dir}/blog"))
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();


    Command::new("ssh-keygen")
        .arg("-R")
        .arg("[localhost]:2222")
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();
}

pub fn teardown() {
    Command::new("docker")
        .arg("compose")
        .arg("down")
        .arg("--remove-orphans")
        .arg("--volumes")
        .status()
        .unwrap()
        .success()
        .then_some(())
        .expect("failed to stop Docker Compose");

    let testdata_dir = "tests/testdata";
    fs::remove_dir_all(testdata_dir).unwrap();
}
