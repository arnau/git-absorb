use git2::{
    AnnotatedCommit, Branch, BranchType, Error as GitError, RebaseOperationType::*, RebaseOptions,
    Reference, Repository,
};

/// Rebases quietly
pub fn run<'a>(
    repo: &'a Repository,
    tip: &'a AnnotatedCommit,
    upstream: &'a AnnotatedCommit,
) -> Result<(), GitError> {
    let mut options = RebaseOptions::new();
    // options.quiet(true).inmemory(true);
    options.inmemory(true);
    let mut operations = repo.rebase(Some(tip), Some(upstream), None, Some(&mut options))?;
    let sig = repo.signature()?;
    dbg!(operations.len());

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

                dbg!(ty, operation.id());

                let c = operations.commit(None, &sig, None)?;
                let comm = repo.find_commit(c)?;
                dbg!(c, comm.message(), comm.parent_id(0)?);
                let head = repo.find_reference("HEAD")?;
                let ann = repo.reference_to_annotated_commit(&head)?;
                dbg!(ann.id());
                let mut direct_ref = head.resolve()?;
                direct_ref.set_target(c, "reflogmsg")?;
            }
            None => {
                return Err(GitError::from_str("Unexpected error while rebasing."));
            }
        }
    }

    operations.finish(None)?;

    Ok(())
}

/// Rebase the `a_ref` on top of `b_ref`.
pub fn experimental_ref<'a>(
    repo: &'a Repository,
    a_ref: &'a Reference,
    b_ref: &'a Reference,
) -> Result<(), GitError> {
    let a_commit = a_ref.peel_to_commit()?;
    let b_commit = b_ref.peel_to_commit()?;
    let a_annot = repo.find_annotated_commit(a_commit.id())?;
    let b_annot = repo.find_annotated_commit(b_commit.id())?;

    let message = format!(
        "Use {} as base for {}",
        b_ref.name().unwrap(),
        a_ref.name().unwrap(),
    );
    dbg!(&message);
    dbg!(&a_commit.id(), &b_commit.id());

    run(repo, &b_annot, &a_annot)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run as rebase;
    use crate::fetch;
    use crate::test as utils;
    use git2::{BranchType, Repository};
    use std::error::Error;
    use tempfile::TempDir;

    #[test]
    fn test_foo() -> Result<(), Box<dyn Error>> {
        let origin_dir = TempDir::new()?;
        let origin = utils::init_repo(origin_dir.path())?;
        let head_target = origin.head()?.target().unwrap();
        let tip = origin.find_commit(head_target)?;

        utils::commit(&origin, "HEAD", "first.txt", "first")?;
        let c2 = utils::commit(&origin, "HEAD", "second.txt", "second")?;

        let local_dir = TempDir::new()?;
        let local = utils::clone_local(&origin, local_dir.path())?;

        let local_head = local.head()?.target().unwrap();
        let local_tip = local.find_commit(local_head)?;

        // Create branch foo
        local.branch("foo", &local_tip, false)?;

        // Do some work in both foo and origin.
        let c3 = utils::commit(&local, "refs/heads/foo", "third.txt", "third")?;
        let c4 = utils::commit(&origin, "HEAD", "tada.txt", "sneaky")?;

        let local_tip = local.find_annotated_commit(local_head)?;

        let origin_head = origin.head()?.target().unwrap();
        let origin_tip = origin.find_annotated_commit(origin_head)?;

        // Main branch
        let tip_branch_name = "main";
        let tip_ref = local.resolve_reference_from_short_name(tip_branch_name)?;
        let tip_refname = tip_ref.name().unwrap();
        let base_refname = local.branch_upstream_name(&tip_refname)?;

        let remote_name = local.branch_upstream_remote(&tip_refname)?;

        // Fetch remote to local main.
        fetch::local(&local, remote_name.as_str().unwrap(), tip_branch_name)?;

        // Find references for main and remote main (i.e. origin/main).
        let tip_ref = local.resolve_reference_from_short_name(tip_branch_name)?;
        let tip_refname = tip_ref.name().unwrap();
        let base_refname = local.branch_upstream_name(&tip_refname)?;
        let base_ref = local.resolve_reference_from_short_name(base_refname.as_str().unwrap())?;

        // Rebase `main` on top of `origin/main`
        crate::rebase::experimental_ref(&local, &tip_ref, &base_ref)?;

        // Find references for `foo` and `main`
        let foo_branch_name = "foo";
        let foo_ref = local.resolve_reference_from_short_name(foo_branch_name)?;
        let foo_refname = foo_ref.name().unwrap();
        let main_branch_name = "main";
        let main_ref = local.resolve_reference_from_short_name(main_branch_name)?;

        // Rebase `foo` on top of `main`
        crate::rebase::experimental_ref(&local, &foo_ref, &main_ref)?;

        // Log stuff
        dbg!("main");
        crate::log::run(&local, &tip_refname);
        dbg!("foo");
        crate::log::run(&local, &foo_refname);

        // dbg!(">>>>>>>>>>>>>>>>>>>>>>>");
        // for branch in local.branches(None)? {
        //     let (branch, branch_type) = branch?;
        //     dbg!(branch.name(), branch_type);
        // }

        // for reference in local.references()? {
        //     dbg!(reference?.name());
        // }

        // crate::log::run(&local, "refs/heads/foo");

        assert_eq!(1, 2);

        Ok(())
    }
}
