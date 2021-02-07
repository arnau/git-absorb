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
                dbg!("exec");
                continue;
            }
            Some(ty) => {
                let idx = repo.index()?;
                if idx.has_conflicts() {
                    operations.abort()?;
                    return Err(GitError::from_str("The rebase operation found conflicts."));
                }

                dbg!(ty);

                operations.commit(None, &sig, None)?;
            }
            None => {
                return Err(GitError::from_str("Unexpected error while rebasing."));
            }
        }
    }

    operations.finish(None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test as utils;
    use git2::Repository;
    use std::error::Error;
    use tempfile::TempDir;

    #[test]
    fn test_foo() -> Result<(), Box<dyn Error>> {
        let origin_dir = TempDir::new()?;
        let origin = utils::init_repo(origin_dir.path())?;
        let head_target = origin.head()?.target().unwrap();
        let tip = origin.find_commit(head_target)?;

        utils::commit(&origin, "HEAD", "first.txt", "first")?;
        utils::commit(&origin, "HEAD", "second.txt", "second")?;

        let head = origin.find_reference("refs/heads/main")?;
        let branch = origin.reference_to_annotated_commit(&head)?;
        let upstream = origin.find_annotated_commit(tip.id())?;

        let local_dir = TempDir::new()?;
        let local = utils::clone_local(&origin, local_dir.path())?;

        let local_head = local.head()?.target().unwrap();
        let local_tip = local.find_commit(local_head)?;
        let local_branch = local.branch("foo", &local_tip, false)?;
        let commit_id3 = utils::commit(&local, "refs/heads/foo", "bar", "third")?;

        // for branch in local.branches(None)? {
        //     let (branch, branch_type) = branch?;
        //     dbg!(branch.name(), branch_type);
        // }

        crate::log::run(&local, "refs/heads/foo");

        assert_eq!(1, 2);

        Ok(())
    }
}
