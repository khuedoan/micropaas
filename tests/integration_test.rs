mod common;

use std::process::Command;

fn push_repo(dir: &str, repo: &str) {
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("local-test")
        .arg(format!("ssh://localhost:2222/{}", repo))
        .current_dir(format!("tests/testdata/{}", dir))
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();

    Command::new("git")
        .arg("push")
        .arg("local-test")
        .current_dir(format!("tests/testdata/{}", dir))
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap()
}

#[test]
fn build_push_deploy() {
    common::setup();
    push_repo("horus", "gitops");
    push_repo("blog", "blog");
    common::teardown();
}
