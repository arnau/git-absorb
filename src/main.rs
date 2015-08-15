//#![deny(warnings)]

extern crate git2;
extern crate docopt;
extern crate rustc_serialize;

use std::str;
use docopt::Docopt;
use git2::{Repository, Error, StatusOptions, ErrorCode, SubmoduleIgnore};

#[derive(RustcDecodable)]
struct Args {
    arg_spec: Vec<String>,
    flag_branch: bool,
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

    loop {
        let statuses = try!(repo.statuses(Some(&mut opts)));

        if args.flag_branch {
            try!(show_branch(&repo));
        }

        print_short(&repo, statuses);

        return Ok(())
    }
}

fn show_branch(repo: &Repository) -> Result<(), Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch ||
                      e.code() == ErrorCode::NotFound => None,
        Err(e) => return Err(e),
    };
    let head = head.as_ref().and_then(|h| h.shorthand());

    println!("## {}", head.unwrap_or("HEAD (no branch)"));

    Ok(())
}

// This version of the output prefixes each path with two status columns and
// shows submodule status information.
fn print_short(repo: &Repository, statuses: git2::Statuses) {
    for entry in statuses.iter().filter(|e| e.status() != git2::STATUS_CURRENT) {
        let mut istatus = match entry.status() {
            s if s.contains(git2::STATUS_INDEX_NEW) => 'A',
            s if s.contains(git2::STATUS_INDEX_MODIFIED) => 'M',
            s if s.contains(git2::STATUS_INDEX_DELETED) => 'D',
            s if s.contains(git2::STATUS_INDEX_RENAMED) => 'R',
            s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => 'T',
            _ => ' ',
        };
        let mut wstatus = match entry.status() {
            s if s.contains(git2::STATUS_WT_NEW) => {
                if istatus == ' ' { istatus = '?'; } '?'
            }
            s if s.contains(git2::STATUS_WT_MODIFIED) => 'M',
            s if s.contains(git2::STATUS_WT_DELETED) => 'D',
            s if s.contains(git2::STATUS_WT_RENAMED) => 'R',
            s if s.contains(git2::STATUS_WT_TYPECHANGE) => 'T',
            _ => ' ',
        };

        if entry.status().contains(git2::STATUS_IGNORED) {
            istatus = '!';
            wstatus = '!';
        }
        if istatus == '?' && wstatus == '?' { continue }
        let mut extra = "";

        // TODO: check for GIT_FILEMODE_COMMIT
        let status = entry.index_to_workdir().and_then(|diff| {
            let ignore = SubmoduleIgnore::Unspecified;
            diff.new_file().path_bytes()
                .and_then(|s| str::from_utf8(s).ok())
                .and_then(|name| repo.submodule_status(name, ignore).ok())
        });
        if let Some(status) = status {
            if status.contains(git2::SUBMODULE_STATUS_WD_MODIFIED) {
                extra = " (new commits)";
            } else if status.contains(git2::SUBMODULE_STATUS_WD_INDEX_MODIFIED) {
                extra = " (modified content)";
            } else if status.contains(git2::SUBMODULE_STATUS_WD_WD_MODIFIED) {
                extra = " (modified content)";
            } else if status.contains(git2::SUBMODULE_STATUS_WD_UNTRACKED) {
                extra = " (untracked content)";
            }
        }

        let (mut a, mut b, mut c) = (None, None, None);
        if let Some(diff) = entry.head_to_index() {
            a = diff.old_file().path();
            b = diff.new_file().path();
        }
        if let Some(diff) = entry.index_to_workdir() {
            a = a.or(diff.old_file().path());
            b = b.or(diff.old_file().path());
            c = diff.new_file().path();
        }

        match (istatus, wstatus) {
            ('R', 'R') => println!("RR {} {} {}{}", a.unwrap().display(),
                                   b.unwrap().display(), c.unwrap().display(),
                                   extra),
            ('R', w) => println!("R{} {} {}{}", w, a.unwrap().display(),
                                 b.unwrap().display(), extra),
            (i, 'R') => println!("{}R {} {}{}", i, a.unwrap().display(),
                                 c.unwrap().display(), extra),
            (i, w) => println!("{}{} {}{}", i, w, a.unwrap().display(), extra),
        }
    }

    for entry in statuses.iter().filter(|e| e.status() == git2::STATUS_WT_NEW) {
        println!("?? {}", entry.index_to_workdir().unwrap().old_file()
                               .path().unwrap().display());
    }
}

// impl Args {
// }

fn main() {
    const USAGE: &'static str = "
usage: absorb [options] [--] [<spec>..]

Options:
    -b, --branch                show branch information
    -h, --help                  show this message
";

    let args = Docopt::new(USAGE).and_then(|d| d.decode())
                                 .unwrap_or_else(|e| e.exit());
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
