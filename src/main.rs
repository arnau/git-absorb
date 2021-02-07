use clap::{AppSettings, Clap};
use git2::{Error as GitError, Repository, StatusOptions};
use std::env;
use std::error::Error;

use absorb::commands::*;
use absorb::rebase;

#[derive(Debug, Clap)]
#[clap(name = "git-absorb", version, global_setting(AppSettings::ColoredHelp))]
struct Cli {
    /// Name of the branch to absorb from.
    #[clap(value_name = "branch", default_value = "main")]
    base_branch: String,
    /// Name of the remote to fetch from
    #[clap(long = "remote", short = 'r', default_value = "origin")]
    remote: String,
    /// Path to the SSH private key. Defaults to ~/.ssh/id_rsa.
    #[clap(long = "private_key", short = 'k')]
    private_key: Option<String>,
}

impl Cli {
    pub fn run(&self) -> Result<String, Box<dyn Error>> {
        let path = "."; // TODO: Add flag to change this.
        let repo = Repository::open(path)?;
        let mut remote = repo.find_remote(&self.remote)?;
        let private_key = match &self.private_key {
            Some(pk) => pk.clone(),
            None => format!("{}/.ssh/id_rsa", env::var("HOME")?),
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

        let head = repo.head()?;
        let head_commit = repo.reference_to_annotated_commit(&head)?;

        // checkout(&self.base_branch);
        let upstream = fetch(&repo, &mut remote, &self.base_branch, &private_key)?;
        let refname = format!("refs/heads/{}", &self.base_branch);
        let base_ref = repo.find_reference(&refname)?;
        let base_commit = repo.reference_to_annotated_commit(&base_ref)?;

        rebase::run(&repo, &base_commit, &upstream)?;

        if &current_branch != &self.base_branch {
            // checkout(&current_branch);
            // rebase(&base_branch);

            rebase::run(&repo, &head_commit, &upstream)?;
        }

        return Ok(format!(
            "Absorbed {}/{} into {}",
            &self.remote, &self.base_branch, &current_branch
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
