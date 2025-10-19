use crate::error::Result;
use git2::{Repository, Signature, RemoteCallbacks, Cred, PushOptions};
use std::path::{Path, PathBuf};
use tracing::{info, debug};

pub struct GitManager {
    github_token: String,
}

impl GitManager {
    pub fn new(github_token: String) -> Self {
        Self { github_token }
    }

    /// Clone a repository to a local path
    pub fn clone_repository(
        &self,
        owner: &str,
        name: &str,
        branch: &str,
        target_dir: &Path,
    ) -> Result<Repository> {
        let repo_url = format!("https://github.com/{}/{}.git", owner, name);

        info!("Cloning repository {} to {:?}", repo_url, target_dir);

        // Setup callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        let token = self.github_token.clone();

        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("x-access-token", &token)
        });

        // Clone options
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.branch(branch);

        let repo = builder.clone(&repo_url, target_dir)?;

        info!("Repository cloned successfully to {:?}", target_dir);

        Ok(repo)
    }

    /// Create a new branch from current HEAD
    pub fn create_branch(&self, repo: &Repository, branch_name: &str) -> Result<()> {
        debug!("Creating branch: {}", branch_name);

        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        repo.branch(branch_name, &commit, false)?;

        // Checkout the new branch
        let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&format!("refs/heads/{}", branch_name))?;

        info!("Branch created and checked out: {}", branch_name);

        Ok(())
    }

    /// Commit all changes
    pub fn commit_changes(
        &self,
        repo: &Repository,
        message: &str,
    ) -> Result<git2::Oid> {
        debug!("Committing changes with message: {}", message);

        let mut index = repo.index()?;

        // Add all changes
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // Get parent commit
        let parent_commit = repo.head()?.peel_to_commit()?;

        // Create signature
        let sig = Signature::now("AutoDev Bot", "autodev@github-actions.bot")?;

        // Create commit
        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent_commit],
        )?;

        info!("Changes committed: {}", commit_id);

        Ok(commit_id)
    }

    /// Push branch to remote
    pub fn push_branch(&self, repo: &Repository, branch_name: &str) -> Result<()> {
        info!("Pushing branch: {}", branch_name);

        let mut remote = repo.find_remote("origin")?;

        // Setup callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        let token = self.github_token.clone();

        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("x-access-token", &token)
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Push the branch
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        remote.push(&[&refspec], Some(&mut push_options))?;

        info!("Branch pushed successfully: {}", branch_name);

        Ok(())
    }

    /// Check if there are any changes in the working directory
    pub fn has_changes(&self, repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(None)?;
        Ok(!statuses.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_manager_creation() {
        let manager = GitManager::new("test_token".to_string());
        assert!(!manager.github_token.is_empty());
    }
}
