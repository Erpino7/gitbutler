use super::*;

#[test]
fn success() {
    let Test {
        project,
        controller,
        ..
    } = &Test::default();

    controller
        .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
        .unwrap();
}

mod error {
    use gitbutler_reference::RemoteRefname;

    use super::*;

    #[test]
    fn missing() {
        let Test {
            project,
            controller,
            ..
        } = &Test::default();

        assert_eq!(
            controller
                .set_base_branch(
                    project,
                    &RemoteRefname::from_str("refs/remotes/origin/missing").unwrap(),
                )
                .unwrap_err()
                .to_string(),
            "remote branch 'refs/remotes/origin/missing' not found"
        );
    }
}

mod go_back_to_workspace {
    use gitbutler_branch::BranchCreateRequest;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_preserve_applied_vbranches() {
        let Test {
            repository,
            project,
            controller,
            ..
        } = &Test::default();

        std::fs::write(repository.path().join("file.txt"), "one").unwrap();
        let oid_one = repository.commit_all("one");
        std::fs::write(repository.path().join("file.txt"), "two").unwrap();
        repository.commit_all("two");
        repository.push();

        controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let vbranch_id = controller
            .create_virtual_branch(project, &BranchCreateRequest::default())
            .unwrap();

        std::fs::write(repository.path().join("another file.txt"), "content").unwrap();
        controller
            .create_commit(project, vbranch_id, "one", None, false)
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert_eq!(branches.len(), 1);

        repository.checkout_commit(oid_one);

        controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].id, vbranch_id);
        assert!(branches[0].active);
    }

    #[test]
    fn from_target_branch_index_conflicts() {
        let Test {
            repository,
            project,
            controller,
            ..
        } = &Test::default();

        std::fs::write(repository.path().join("file.txt"), "one").unwrap();
        let oid_one = repository.commit_all("one");
        std::fs::write(repository.path().join("file.txt"), "two").unwrap();
        repository.commit_all("two");
        repository.push();

        controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert!(branches.is_empty());

        repository.checkout_commit(oid_one);
        std::fs::write(repository.path().join("file.txt"), "tree").unwrap();

        assert!(matches!(
            controller
                .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
                .unwrap_err()
                .downcast_ref(),
            Some(Marker::ProjectConflict)
        ));
    }

    #[test]
    fn from_target_branch_with_uncommited() {
        let Test {
            repository,
            project,
            controller,
            ..
        } = &Test::default();

        std::fs::write(repository.path().join("file.txt"), "one").unwrap();
        let oid_one = repository.commit_all("one");
        std::fs::write(repository.path().join("file.txt"), "two").unwrap();
        repository.commit_all("two");
        repository.push();

        controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert!(branches.is_empty());

        repository.checkout_commit(oid_one);
        std::fs::write(repository.path().join("another file.txt"), "tree").unwrap();

        assert!(matches!(
            controller
                .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
                .unwrap_err()
                .downcast_ref(),
            Some(Marker::ProjectConflict)
        ));
    }

    #[test]
    fn from_target_branch_with_commit() {
        let Test {
            repository,
            project,
            controller,
            ..
        } = &Test::default();

        std::fs::write(repository.path().join("file.txt"), "one").unwrap();
        let oid_one = repository.commit_all("one");
        std::fs::write(repository.path().join("file.txt"), "two").unwrap();
        repository.commit_all("two");
        repository.push();

        let base = controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert!(branches.is_empty());

        repository.checkout_commit(oid_one);
        std::fs::write(repository.path().join("another file.txt"), "tree").unwrap();
        repository.commit_all("three");

        let base_two = controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert_eq!(branches.len(), 0);
        assert_eq!(base_two, base);
    }

    #[test]
    fn from_target_branch_without_any_changes() {
        let Test {
            repository,
            project,
            controller,
            ..
        } = &Test::default();

        std::fs::write(repository.path().join("file.txt"), "one").unwrap();
        let oid_one = repository.commit_all("one");
        std::fs::write(repository.path().join("file.txt"), "two").unwrap();
        repository.commit_all("two");
        repository.push();

        let base = controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert!(branches.is_empty());

        repository.checkout_commit(oid_one);

        let base_two = controller
            .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
            .unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert_eq!(branches.len(), 0);
        assert_eq!(base_two, base);
    }
}
