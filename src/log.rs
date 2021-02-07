// source: https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs

use git2::Error;
use git2::{Commit, Repository};
use std::str;

pub fn run(repo: &Repository, refname: &str) -> Result<(), Error> {
    let mut revwalk = repo.revwalk()?;
    let base = git2::Sort::NONE;

    revwalk.set_sorting(base | git2::Sort::TIME)?;

    if refname == "HEAD" {
        revwalk.push_head()?;
    } else {
        let oid = repo.refname_to_id(refname)?;
        revwalk.push(oid)?;
    }

    // Filter our revwalk based on the CLI parameters
    macro_rules! filter_try {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }
        };
    }
    let revwalk = revwalk.filter_map(|id| {
        let id = filter_try!(id);
        let commit = filter_try!(repo.find_commit(id));
        let parents = commit.parents().len();
        if parents < 1 {
            return None;
        }

        Some(Ok(commit))
    });

    // print!
    for commit in revwalk {
        let commit = commit?;
        print_commit(&commit);
    }

    Ok(())
}

fn print_commit(commit: &Commit) {
    println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);

    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}
