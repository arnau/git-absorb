extern crate absorb;
use absorb::commands::*;

extern crate git2;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use git2::{Repository, Error, StatusOptions};


#[derive(RustcDecodable)]
struct Args {
    arg_spec: Vec<String>,
    flag_git_dir: Option<String>,
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
