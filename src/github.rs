use crate::models::{Branch, BranchInfo, CommitWrapper, PullRequest, Repository};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct CheckSuiteResponse {
    check_suites: Vec<CheckSuite>,
}

#[derive(Deserialize)]
struct CheckSuite {
    conclusion: Option<String>,
}

pub fn fetch_repos(client: &Client, token: &str) -> Vec<Repository> {
    let url = "https://api.github.com/user/repos?per_page=15&sort=updated";
    let res = client
        .get(url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    match res {
        Ok(response) if response.status().is_success() => response.json().unwrap_or_default(),
        _ => vec![],
    }
}

pub fn fetch_branch_details(
    client: &Client,
    token: &str,
    owner: &str,
    repo_name: &str,
) -> Vec<BranchInfo> {
    let branches_url = format!(
        "https://api.github.com/repos/{}/{}/branches",
        owner, repo_name
    );
    let res = client
        .get(&branches_url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    let mut result = Vec::new();

    let branches: Vec<Branch> = match res {
        Ok(resp) if resp.status().is_success() => resp.json().unwrap_or_default(),
        _ => return result,
    };

    for branch in branches {
        // 1. Commits récents
        let commits_url = format!(
            "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=2",
            owner, repo_name, branch.name
        );
        let commits_res = client
            .get(&commits_url)
            .header("User-Agent", "github-workflow-rust")
            .header("Authorization", format!("Bearer {}", token))
            .send();

        let commits: Vec<CommitWrapper> = match commits_res {
            Ok(resp) if resp.status().is_success() => resp.json().unwrap_or_default(),
            _ => vec![],
        };

        // 2. Pull Requests associées à la branche
        let prs_url = format!(
            "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=open",
            owner, repo_name, owner, branch.name
        );
        let prs_res = client
            .get(&prs_url)
            .header("User-Agent", "github-workflow-rust")
            .header("Authorization", format!("Bearer {}", token))
            .send();

        let prs: Vec<PullRequest> = match prs_res {
            Ok(resp) if resp.status().is_success() => resp.json().unwrap_or_default(),
            _ => vec![],
        };

        // 3. Statut CI/CD (GitHub Actions Check Suites)
        let ci_url = format!(
            "https://api.github.com/repos/{}/{}/check-suites?ref={}",
            owner, repo_name, branch.name
        );
        let ci_res = client
            .get(&ci_url)
            .header("User-Agent", "github-workflow-rust")
            .header("Authorization", format!("Bearer {}", token))
            .send();

        let mut ci_status = "unknown".to_string();
        if let Ok(resp) = ci_res {
            if resp.status().is_success() {
                if let Ok(ci_data) = resp.json::<CheckSuiteResponse>() {
                    if let Some(suite) = ci_data.check_suites.first() {
                        if let Some(conclusion) = &suite.conclusion {
                            ci_status = conclusion.clone(); // ex: "success", "failure"
                        }
                    }
                }
            }
        }

        result.push(BranchInfo {
            branch,
            commits,
            prs,
            ci_status,
        });
    }

    result
}
