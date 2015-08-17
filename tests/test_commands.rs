use absorb::commands::*;
use std::process::Command;
use git2::{Repository};

fn git_revparse() -> String {
  let output = Command::new("git")
                       .arg("rev-parse")
                       .arg("--abbrev-ref")
                       .arg("HEAD")
                       .output()
                       .unwrap_or_else(|e| {
                          panic!("failed to execute process: {}", e) });

  String::from_utf8_lossy(&output.stdout).trim_right().to_string()
}

#[test]
fn test_branch() {
    let repo = Repository::open(".").unwrap();
    let branchname = branch(&repo).unwrap();
    let contrast = git_revparse();

    assert_eq!(contrast, branchname);
}
