use clap::{AppSettings, Clap};
use git2::{BranchType, Error as GitError, Repository, RepositoryState, StatusOptions};
use std::error::Error;

mod commands;
use commands::*;

fn resolve_base_branch(repo: &Repository) -> Result<String, Box<dyn Error>> {
    let default_branches = vec!["main", "master", "dev"];

    for branch in default_branches {
        if let Ok(branch) = repo.find_branch(branch, BranchType::Local) {
            return Ok(branch.name()?.expect("a valid UTF-8 branch name").into());
        }
    }

    if let Some(branch) = repo.config()?.get_entry("init.defaultbranch")?.value() {
        Ok(branch.into())
    } else {
        Err(
            Box::new(
            GitError::from_str("Couldn't guess the base branch. Please provide the base branch using the `base_branch` flag.".into())
            )
        )
    }
}

#[derive(Debug, Clap)]
#[clap(name = "git-absorb", version, global_setting(AppSettings::ColoredHelp))]
struct Cli {
    /// Name of the base branch.
    #[clap(long = "base_branch", short = 'b')]
    base_branch: Option<String>,
    /// Name of the remote to fetch from
    #[clap(long = "remote", short = 'r', default_value = "origin")]
    remote: String,
}

impl Cli {
    pub fn run(&self) -> Result<String, Box<dyn Error>> {
        let path = "."; // TODO: Add flag to change this.
        let repo = Repository::open(path)?;
        let base_branch = if let Some(branch) = &self.base_branch {
            branch.into()
        } else {
            resolve_base_branch(&repo)?
        };

        if repo.is_bare() {
            return Err(Box::new(GitError::from_str(
                "Bare repositories are not allowed",
            )));
        }

        let current_branch = branch(&repo)?;

        let mut opts = StatusOptions::new();
        let statuses = repo.statuses(Some(&mut opts))?;

        if !statuses.is_empty() {
            return Err(Box::new(GitError::from_str(
                "The working directory is not in a clean state.",
            )));
        }

        checkout(&base_branch);
        pull(&self.remote, "master");

        if &current_branch != &base_branch {
            checkout(&current_branch);
            rebase(&base_branch);
        }

        return Ok(format!(
            "Absorbed {} into {}",
            &base_branch, &current_branch
        ));
    }
}

fn main() {
    let cli: Cli = Cli::parse();

    match cli.run() {
        Ok(msg) => {
            println!("{}", msg);
        }

        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
