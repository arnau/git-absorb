// Source: https://github.com/rust-lang/git2-rs/blob/master/src/test.rs

use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use git2::{Branch, Oid, Repository, RepositoryInitOptions};

/// Creates a dummy repo.
pub fn init_repo<P: AsRef<Path>>(path: P) -> Result<Repository, Box<dyn Error>> {
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");

    let repo = Repository::init_opts(path, &opts)?;

    {
        let mut config = repo.config()?;
        config.set_str("user.name", "name")?;
        config.set_str("user.email", "email")?;

        let mut index = repo.index()?;
        let id = index.write_tree()?;

        let tree = repo.find_tree(id)?;
        let sig = repo.signature()?;

        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
    }

    Ok(repo)
}

pub fn clone_local<P: AsRef<Path>>(
    origin: &Repository,
    target: P,
) -> Result<Repository, Box<dyn Error>> {
    let origin_path = match origin.workdir() {
        None => origin.path(),
        Some(path) => path,
    };

    let repo = Repository::clone(
        origin_path
            .to_str()
            .expect("Expected a path with a UTF-8 representation."),
        target,
    )?;

    Ok(repo)
}

pub fn commit(
    repo: &Repository,
    refname: &str,
    filename: &str,
    message: &str,
) -> Result<Oid, Box<dyn Error>> {
    let mut index = repo.index()?;
    let root = repo
        .path()
        .parent()
        .ok_or(git2::Error::from_str("no parent"))?;

    File::create(&root.join(filename))?;
    index.add_path(Path::new(filename))?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let sig = repo.signature()?;
    let head_id = repo.refname_to_id(refname)?;

    let parent = repo.find_commit(head_id)?;
    let commit = repo.commit(Some(refname), &sig, &sig, message, &tree, &[&parent])?;

    Ok(commit)
}

// pub fn path2url(path: &Path) -> String {
//     Url::from_file_path(path).unwrap().to_string()
// }

pub fn worktrees_env_init(repo: &Repository) -> (TempDir, Branch<'_>) {
    let oid = repo.head().unwrap().target().unwrap();
    let commit = repo.find_commit(oid).unwrap();
    let branch = repo.branch("wt-branch", &commit, true).unwrap();
    let wtdir = TempDir::new().unwrap();
    (wtdir, branch)
}
