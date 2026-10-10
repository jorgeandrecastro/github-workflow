use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Repository {
    pub name: String,
    pub owner: Owner,
    pub html_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Owner {
    pub login: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Branch {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommitWrapper {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommitDetails {
    pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
}

#[derive(Clone)]
pub enum BranchState {
    NotLoaded,
    Loading,
    Loaded(Vec<BranchInfo>),
}

#[derive(Clone)]
pub struct BranchInfo {
    pub branch: Branch,
    pub commits: Vec<CommitWrapper>,
    pub prs: Vec<PullRequest>,
    pub ci_status: String, // "success", "failure", ou "unknown"
}

#[derive(Clone)]
pub struct RepoItem {
    pub repo: Repository,
    pub is_expanded: bool,
    pub branches_state: BranchState,
}

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
