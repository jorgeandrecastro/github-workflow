use serde::Deserialize;

/// Represents a GitHub repository.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Repository {
    pub name: String,
    pub owner: Owner,
    pub html_url: String,
}

/// Represents the owner of a repository.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Owner {
    pub login: String,
}

/// Represents a Git branch.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Branch {
    pub name: String,
}

/// Wrapper for commit data from GitHub API.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CommitWrapper {
    pub sha: String,
    pub commit: CommitDetails,
}

/// Details of a specific commit.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CommitDetails {
    pub message: String,
}

/// Represents an open Pull Request.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
}

/// State of branch loading for a repository (Lazy Loading).
#[derive(Clone, Debug, PartialEq)]
pub enum BranchState {
    NotLoaded,
    Loading,
    Loaded(Vec<BranchInfo>),
}

/// Detailed information about a branch including commits, PRs, and CI status.
#[derive(Clone, Debug, PartialEq)]
pub struct BranchInfo {
    pub branch: Branch,
    pub commits: Vec<CommitWrapper>,
    pub prs: Vec<PullRequest>,
    pub ci_status: String, // "success", "failure", "in_progress", or "unknown"
}

/// Represents a repository item in the TUI state.
#[derive(Clone, Debug, PartialEq)]
pub struct RepoItem {
    pub repo: Repository,
    pub is_expanded: bool,
    pub branches_state: BranchState,
}

/// Maps the flattened TUI list items back to their original hierarchical data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayItem {
    Repo {
        repo_idx: usize,
    },
    Branch {
        repo_idx: usize,
        branch_idx: usize,
    },
    Commit {
        repo_idx: usize,
        branch_idx: usize,
        commit_idx: usize,
    },
    PullRequest {
        repo_idx: usize,
        branch_idx: usize,
        pr_idx: usize,
    },
    Loading,
    Empty,
}