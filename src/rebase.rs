use git2::{AnnotatedCommit, Error as GitError, RebaseOperationType::*, RebaseOptions, Repository};

/// Rebases quietly
pub fn run<'a>(
    repo: &'a Repository,
    local: &'a AnnotatedCommit,
    upstream: &'a AnnotatedCommit,
) -> Result<(), GitError> {
    let mut options = RebaseOptions::new();
    options.quiet(true).inmemory(true);
    let mut operations = repo.rebase(Some(local), Some(upstream), None, Some(&mut options))?;
    let sig = repo.signature()?;

    while let Some(operation) = operations.next() {
        let operation = operation?;
        match operation.kind() {
            Some(Exec) => {
                continue;
            }
            _ => {
                let idx = repo.index()?;
                if idx.has_conflicts() {
                    operations.abort()?;
                    return Err(GitError::from_str("The rebase operation found conflicts."));
                }

                operations.commit(None, &sig, None)?;
            }
        }
    }

    operations.finish(None)?;

    Ok(())
}
