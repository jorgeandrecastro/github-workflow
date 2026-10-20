use crate::models::{Branch, BranchInfo, CommitWrapper, PullRequest, Repository};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
struct CheckSuiteResponse {
    check_suites: Vec<CheckSuite>,
}

#[derive(Deserialize)]
struct CheckSuite {
    status: Option<String>,
    conclusion: Option<String>,
}

/// Fetches the authenticated user's recent repositories.
pub fn fetch_repos(client: &Client, token: &str) -> Vec<Repository> {
    let url = "https://api.github.com/user/repos?per_page=15&sort=updated";
    let res = client
        .get(url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    match res {
        Ok(response) if response.status().is_success() => {
            response.json::<Vec<Repository>>().unwrap_or_default()
        }
        _ => vec![],
    }
}

/// Fetches branch details, recent commits, open pull requests, and CI/CD status in parallel.
pub fn fetch_branch_details(
    client: &Client,
    token: &str,
    owner: &str,
    repo_name: &str,
) -> Vec<BranchInfo> {
    let branches_url = format!(
        "https://api.github.com/repos/{}/{}/branches?per_page=10",
        owner, repo_name
    );
    let res = client
        .get(&branches_url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    let branches: Vec<Branch> = match res {
        Ok(resp) if resp.status().is_success() => resp.json().unwrap_or_default(),
        _ => return vec![],
    };

    // Parallel fetch via scoped threads
    let results: Vec<BranchInfo> = thread::scope(|s| {
        let handles: Vec<_> = branches
            .into_iter()
            .map(|branch| {
                let client_ref = client;
                let token_ref = token;
                let owner_ref = owner;
                let repo_ref = repo_name;

                s.spawn(move || {
                    let branch_name = &branch.name;

                    // 1. Fetch recent commits
                    let commits_url = format!(
                        "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=2",
                        owner_ref, repo_ref, branch_name
                    );
                    let commits: Vec<CommitWrapper> = client_ref
                        .get(&commits_url)
                        .header("User-Agent", "github-workflow-rust")
                        .header("Authorization", format!("Bearer {}", token_ref))
                        .send()
                        .and_then(|r| r.json())
                        .unwrap_or_default();

                    // 2. Fetch open Pull Requests
                    let prs_url = format!(
                        "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=open",
                        owner_ref, repo_ref, owner_ref, branch_name
                    );
                    let prs: Vec<PullRequest> = client_ref
                        .get(&prs_url)
                        .header("User-Agent", "github-workflow-rust")
                        .header("Authorization", format!("Bearer {}", token_ref))
                        .send()
                        .and_then(|r| r.json())
                        .unwrap_or_default();

                    // 3. Fetch CI/CD status
                    let ci_url = format!(
                        "https://api.github.com/repos/{}/{}/check-suites?ref={}",
                        owner_ref, repo_ref, branch_name
                    );
                    let mut ci_status = "unknown".to_string();

                    if let Ok(resp) = client_ref
                        .get(&ci_url)
                        .header("User-Agent", "github-workflow-rust")
                        .header("Authorization", format!("Bearer {}", token_ref))
                        .send()
                    {
                        if resp.status().is_success() {
                            if let Ok(ci_data) = resp.json::<CheckSuiteResponse>() {
                                if let Some(suite) = ci_data.check_suites.first() {
                                    if let Some(conclusion) = &suite.conclusion {
                                        ci_status = conclusion.clone();
                                    } else if let Some(status) = &suite.status {
                                        ci_status = status.clone(); // In-progress build status
                                    }
                                }
                            }
                        }
                    }

                    BranchInfo {
                        branch,
                        commits,
                        prs,
                        ci_status,
                    }
                })
            })
            .collect();

        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });

    results
}