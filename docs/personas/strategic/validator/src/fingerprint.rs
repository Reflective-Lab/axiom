//! Git Repository Fingerprinting
//!
//! Provides artifact provenance tracking through git commit hashing and dirty tree detection.
//!
//! This module enables drift detection to link findings to specific commit hashes,
//! satisfying DRIFT-03 requirement (each finding links to commit hash).

use std::path::Path;

use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};

/// Git repository fingerprint capturing commit state and uncommitted changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFingerprint {
    /// HEAD commit hash if repository is valid
    pub commit_hash: Option<String>,
    /// Whether working tree has uncommitted changes to tracked files
    pub dirty: bool,
    /// Current tree hash if dirty (computed from index)
    pub tree_hash: Option<String>,
}

/// Fingerprint a git repository at the given path
///
/// Returns fingerprint containing HEAD commit hash, dirty flag, and tree hash if dirty.
///
/// Dirty detection includes only tracked file changes (excludes untracked files and submodules).
///
/// # Errors
///
/// Returns `git2::Error` if repository cannot be opened or git operations fail.
pub fn fingerprint_repo(repo_path: &Path) -> Result<ArtifactFingerprint, git2::Error> {
    let repo = Repository::open(repo_path)?;

    // Get HEAD commit hash
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let commit_hash = commit.id().to_string();

    // Check for dirty tree (tracked files only, exclude submodules)
    let mut opts = StatusOptions::new();
    opts.include_untracked(false); // Ignore untracked files
    opts.exclude_submodules(true); // Ignore submodules

    let statuses = repo.statuses(Some(&mut opts))?;
    let dirty = !statuses.is_empty();

    // If dirty, compute tree hash from index
    let tree_hash = if dirty {
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        Some(tree_id.to_string())
    } else {
        None
    };

    Ok(ArtifactFingerprint {
        commit_hash: Some(commit_hash),
        dirty,
        tree_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_current_repo() {
        // Test on the current repository (converge-personas)
        let repo_path = Path::new("../..");
        let fingerprint = fingerprint_repo(repo_path)
            .expect("Should successfully fingerprint converge-personas repo");

        // Verify commit hash is present and non-empty
        assert!(
            fingerprint.commit_hash.is_some(),
            "Commit hash should be present"
        );
        let commit_hash = fingerprint.commit_hash.unwrap();
        assert!(
            !commit_hash.is_empty(),
            "Commit hash should not be empty"
        );
        assert_eq!(
            commit_hash.len(),
            40,
            "Commit hash should be 40 characters (SHA-1)"
        );

        // Don't assert specific dirty state as it may vary during development

        // If dirty, tree_hash should be present
        if fingerprint.dirty {
            assert!(
                fingerprint.tree_hash.is_some(),
                "Tree hash should be present when dirty"
            );
        }
    }
}
