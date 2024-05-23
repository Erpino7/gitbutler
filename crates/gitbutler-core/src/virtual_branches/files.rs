use std::path;

use anyhow::{Context, Result};
use serde::Serialize;

use super::errors;
use crate::git::{self, diff};

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchFile {
    pub path: path::PathBuf,
    pub hunks: Vec<diff::GitHunk>,
    pub binary: bool,
}

pub fn list_remote_commit_files(
    repository: &git::Repository,
    commit_oid: git::Oid,
) -> Result<Vec<RemoteBranchFile>, errors::ListRemoteCommitFilesError> {
    let commit = match repository.find_commit(commit_oid) {
        Ok(commit) => Ok(commit),
        Err(git::Error::NotFound(_)) => Err(errors::ListRemoteCommitFilesError::CommitNotFound(
            commit_oid,
        )),
        Err(error) => Err(errors::ListRemoteCommitFilesError::Other(error.into())),
    }?;

    if commit.parent_count() == 0 {
        return Ok(vec![]);
    }

    let commit_tree = commit.tree().context("failed to get commit tree")?;

    // if we have a conflicted commit, we just need a vector of the files that conflicted
    if commit.is_conflicted() {
        let conflict_files_list = commit_tree.get_name(".conflict-files").unwrap();
        let files_list = conflict_files_list.to_object(repository).unwrap();
        let list = files_list.as_blob().context("failed to get conflict files list")?;
        // split this list blob into lines and return a Vec of RemoteBranchFile
        let files = list
            .content()
            .split(|&byte| byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| {
                let path = path::PathBuf::from(std::str::from_utf8(line).unwrap());
                RemoteBranchFile {
                    path,
                    hunks: vec![],
                    binary: false,
                }
            })
            .collect();
        return Ok(files);
    }

    let parent = commit.parent(0).context("failed to get parent commit")?;
    let parent_tree = parent.tree().context("failed to get parent tree")?;
    let diff_files = diff::trees(repository, &parent_tree, &commit_tree)?;

    Ok(diff_files
        .into_iter()
        .map(|(path, file)| {
            let binary = file.hunks.iter().any(|h| h.binary);
            RemoteBranchFile {
                path,
                hunks: file.hunks,
                binary,
            }
        })
        .collect())
}
