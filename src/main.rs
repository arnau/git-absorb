//#![deny(warnings)]

extern crate git2;
extern crate docopt;
extern crate rustc_serialize;

use std::process::{Command, exit};

use docopt::Docopt;
use git2::{Repository, Error, StatusOptions, ErrorCode};




#[derive(RustcDecodable)]
struct Args {
    arg_spec: Vec<String>,
    flag_git_dir: Option<String>,
}

/// Gets the current branch.
///
/// # Examples
///
/// ```
/// let repo = try!(Repository::open("."));
/// let branchname = try!(branch(&repo));
/// ```
fn branch<'a>(repo: &'a Repository) -> Result<String, Error> {
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


fn run(args: &Args) -> Result<(), Error> {
    let path = args.flag_git_dir.clone().unwrap_or(".".to_string());
    let repo = try!(Repository::open(&path));

    if repo.is_bare() {
        return Err(Error::from_str("Bare repositories are not allowed"))
    }

    let mut opts = StatusOptions::new();

    for spec in args.arg_spec.iter() {
        opts.pathspec(spec);
    }

    let current_branch = try!(branch(&repo));

    println!("Current branch {:?}", current_branch);

    checkout("master");
    pull("origin", "master");
    checkout(&current_branch);
    rebase("master");

    return Ok(())
}

fn checkout(target_branch: &str) {
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

fn pull(target_remote: &str, target_branch: &str) {
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

fn rebase(target_branch: &str) {
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

fn main() {
    const USAGE: &'static str = "
usage: absorb [options] [--] [<branchname>]

Options:
    -h, --help                  show this message
";

    let args = Docopt::new(USAGE).and_then(|d| d.decode())
                                 .unwrap_or_else(|e| e.exit());
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
