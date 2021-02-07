use git2::{
    AnnotatedCommit, AutotagOption, Error as GitError, FetchOptions, Progress, Remote,
    RemoteCallbacks, Repository,
};
use std::io::{self, Write};
use std::str;

fn progress_printer<'a>(stats: Progress<'a>) -> bool {
    if stats.received_objects() == stats.total_objects() {
        println!(
            "Resolving deltas {}/{}",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else if stats.total_objects() > 0 {
        println!(
            "Received {}/{} objects ({}) in {} bytes",
            stats.received_objects(),
            stats.total_objects(),
            stats.indexed_objects(),
            stats.received_bytes()
        );
    }
    io::stdout().flush().unwrap();
    true
}

/// Fetches the given remote reference.
pub fn run<'a>(
    repo: &'a Repository,
    remote: &'a mut Remote,
    reference: &str,
    private_key: &str,
    passphrase: Option<String>,
) -> Result<AnnotatedCommit<'a>, GitError> {
    let remote_name = remote.name().expect("remote name to exist");
    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(progress_printer);
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(private_key),
            passphrase.as_ref().map(String::as_str),
        )
    });

    let mut options = FetchOptions::new();
    options
        .remote_callbacks(callbacks)
        .download_tags(AutotagOption::All);

    println!("Fetching {}/{}", remote_name, reference);

    remote.fetch(&[reference], Some(&mut options), None)?;

    let stats = remote.stats();
    println!(
        "Received {}/{} objects in {} bytes",
        stats.indexed_objects(),
        stats.total_objects(),
        stats.received_bytes()
    );

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}
