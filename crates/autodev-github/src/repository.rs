use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub branch: String,
}

impl Repository {
    pub fn new(owner: String, name: String) -> Self {
        Self {
            owner,
            name,
            branch: "main".to_string(),
        }
    }

    pub fn with_branch(mut self, branch: String) -> Self {
        self.branch = branch;
        self
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.name)
    }

    pub fn ssh_url(&self) -> String {
        format!("git@github.com:{}/{}.git", self.owner, self.name)
    }

    pub fn https_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }

    pub fn actions_url(&self) -> String {
        format!("https://github.com/{}/{}/actions", self.owner, self.name)
    }
}

impl std::fmt::Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        let repo = Repository::new("owner".to_string(), "name".to_string());
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "name");
        assert_eq!(repo.branch, "main");
    }

    #[test]
    fn test_repository_urls() {
        let repo = Repository::new("myorg".to_string(), "myrepo".to_string());

        assert_eq!(repo.full_name(), "myorg/myrepo");
        assert_eq!(repo.clone_url(), "https://github.com/myorg/myrepo.git");
        assert_eq!(repo.ssh_url(), "git@github.com:myorg/myrepo.git");
        assert_eq!(repo.https_url(), "https://github.com/myorg/myrepo");
    }

    #[test]
    fn test_with_branch() {
        let repo = Repository::new("owner".to_string(), "name".to_string())
            .with_branch("develop".to_string());
        assert_eq!(repo.branch, "develop");
    }
}