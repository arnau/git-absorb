extern crate git2;

use std::process::{Command, exit};
use self::git2::{Repository, Error, ErrorCode};

/// Gets the current branch.
///
/// # Examples
///
/// ```
/// let repo = try!(Repository::open("."));
/// let branchname = try!(branch(&repo));
/// ```
pub fn branch<'a>(repo: &'a Repository) -> Result<String, Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
            Err(ref e) if e.code() == ErrorCode::UnbornBranch ||
                          e.code() == ErrorCode::NotFound
                       => None,
            Err(e) => return Err(e),
    };
    let branchref = head.as_ref().and_then(|h| h.shorthand());
    let branchname = branchref.unwrap_or("HEAD (no branch)").to_string();

    Ok(branchname)
}

pub fn checkout(target_branch: &str) {
    let output = Command::new("git")
                         .arg("checkout")
                         .arg(target_branch)
                         .output()
                         .unwrap_or_else(|e| {
                            panic!("failed to execute process: {}", e) });
    let status_code = output.status.code();

    println!("Checking out {} ...", target_branch);

    match status_code {
        Some(0) => println!("{}", String::from_utf8_lossy(&output.stdout)),
        Some(_) => {
             println!("{}", String::from_utf8_lossy(&output.stderr));
             exit(status_code.unwrap());
        }
        None => println!("Something wrong happened"),
    }
}

pub fn pull(target_remote: &str, target_branch: &str) {
    let output = Command::new("git")
                         .arg("pull")
                         .arg("--quiet")
                         .arg("--rebase=preserve")
                         .arg(target_remote)
                         .arg(target_branch)
                         .output()
                         .unwrap_or_else(|e| {
                            panic!("failed to execute process: {}", e) });
    let status_code = output.status.code();

    println!("Pulling {} {} ...", target_remote, target_branch);

    match status_code {
        Some(0) => println!("{}", String::from_utf8_lossy(&output.stdout)),
        Some(_) => {
             println!("{}", String::from_utf8_lossy(&output.stderr));
             exit(status_code.unwrap());
        }
        None => println!("Something wrong happened"),
    }
}

pub fn rebase(target_branch: &str) {
    let output = Command::new("git")
                         .arg("rebase")
                         .arg(target_branch)
                         .output()
                         .unwrap_or_else(|e| {
                            panic!("failed to execute process: {}", e) });
    let status_code = output.status.code();

    println!("Rebasing {} ...", target_branch);

    match status_code {
        Some(0) => println!("{}", String::from_utf8_lossy(&output.stdout)),
        Some(_) => {
             println!("{}", String::from_utf8_lossy(&output.stderr));
             exit(status_code.unwrap());
        }
        None => println!("Something wrong happened"),
    }
}
