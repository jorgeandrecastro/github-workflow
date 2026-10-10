use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{env, io, time::Duration};

#[derive(Deserialize, Debug, Clone)]
struct Repository {
    name: String,
    owner: Owner,
    html_url: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Owner {
    login: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Branch {
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CommitWrapper {
    sha: String,
    commit: CommitDetails,
}

#[derive(Deserialize, Debug, Clone)]
struct CommitDetails {
    message: String,
}

struct RepoItem {
    repo: Repository,
    is_expanded: bool,
    branches: Option<Vec<(Branch, Vec<CommitWrapper>)>>,
}

// Représente les lignes affichées à l'écran pour mapper facilement la sélection clavier
enum DisplayItem {
    Repo { repo_idx: usize },
    Branch { repo_idx: usize, branch_idx: usize },
    Commit { repo_idx: usize, branch_idx: usize, commit_idx: usize },
    Loading,
    Empty,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = match env::var("GITHUB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Erreur : La variable GITHUB_TOKEN n'est pas définie.");
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let url = "https://api.github.com/user/repos?per_page=15&sort=updated";

    let res = client
        .get(url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    let repos: Vec<Repository> = if res.status().is_success() {
        res.json().unwrap_or_default()
    } else {
        vec![]
    };

    let mut repo_items: Vec<RepoItem> = repos
        .into_iter()
        .map(|repo| RepoItem {
            repo,
            is_expanded: false,
            branches: None,
        })
        .collect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    if !repo_items.is_empty() {
        list_state.select(Some(0));
    }

    loop {
        let mut display_map: Vec<DisplayItem> = Vec::new();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let header = Paragraph::new(" 🚀 GITHUB-WORKFLOW - Dashboard TUI ")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Projet"));
            f.render_widget(header, chunks[0]);

            let mut items: Vec<ListItem> = Vec::new();

            for (repo_idx, item) in repo_items.iter().enumerate() {
                display_map.push(DisplayItem::Repo { repo_idx });
                
                let prefix = if item.is_expanded { "📂 [▼]" } else { "📁 [▶]" };
                let title = format!("{} {} ({})", prefix, item.repo.name, item.repo.html_url);
                
                let is_current_selected = list_state.selected() == Some(display_map.len() - 1);
                let style = if is_current_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                items.push(ListItem::new(title).style(style));

                if item.is_expanded {
                    if let Some(ref branches) = item.branches {
                        if branches.is_empty() {
                            display_map.push(DisplayItem::Empty);
                            items.push(ListItem::new("     (Aucune branche trouvée)").style(Style::default().fg(Color::DarkGray)));
                        } else {
                            for (branch_idx, (branch, commits)) in branches.iter().enumerate() {
                                display_map.push(DisplayItem::Branch { repo_idx, branch_idx });
                                
                                let is_b_selected = list_state.selected() == Some(display_map.len() - 1);
                                let b_style = if is_b_selected {
                                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                                } else {
                                    Style::default().fg(Color::LightYellow)
                                };

                                items.push(ListItem::new(format!("     ├── 🌿 Branche : {}", branch.name)).style(b_style));

                                for (commit_idx, commit) in commits.iter().enumerate() {
                                    display_map.push(DisplayItem::Commit { repo_idx, branch_idx, commit_idx });
                                    let short_sha = if commit.sha.len() >= 7 { &commit.sha[..7] } else { &commit.sha };
                                    let msg = commit.commit.message.lines().next().unwrap_or("");
                                    
                                    let is_c_selected = list_state.selected() == Some(display_map.len() - 1);
                                    let c_style = if is_c_selected {
                                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                                    } else {
                                        Style::default().fg(Color::DarkGray)
                                    };

                                    items.push(ListItem::new(format!("     │    └── 💻 [{}] {}", short_sha, msg)).style(c_style));
                                }
                            }
                        }
                    } else {
                        display_map.push(DisplayItem::Loading);
                        items.push(ListItem::new("     Chargement des branches...").style(Style::default().fg(Color::Gray)));
                    }
                }
            }

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Dépôts GitHub récents"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            f.render_stateful_widget(list, chunks[1], &mut list_state);

            let help = Paragraph::new(" Naviguer: ↑/↓ | Dérouler: Espace/Entrée | Ouvrir sur le Web: o | Quitter: q ")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title(" Raccourcis "));
            f.render_widget(help, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down => {
                        if let Some(selected) = list_state.selected() {
                            if selected < display_map.len().saturating_sub(1) {
                                list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = list_state.selected() {
                            if selected > 0 {
                                list_state.select(Some(selected - 1));
                            }
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(selected) = list_state.selected() {
                            if let Some(DisplayItem::Repo { repo_idx }) = display_map.get(selected) {
                                let item = &mut repo_items[*repo_idx];
                                item.is_expanded = !item.is_expanded;

                                if item.is_expanded && item.branches.is_none() {
                                    let branches = fetch_branches_and_commits(
                                        &client,
                                        &token,
                                        &item.repo.owner.login,
                                        &item.repo.name,
                                    );
                                    item.branches = Some(branches);
                                }
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        if let Some(selected) = list_state.selected() {
                            if let Some(display_item) = display_map.get(selected) {
                                match display_item {
                                    DisplayItem::Repo { repo_idx } => {
                                        let url = &repo_items[*repo_idx].repo.html_url;
                                        let _ = open::that(url);
                                    }
                                    DisplayItem::Branch { repo_idx, branch_idx } => {
                                        let repo = &repo_items[*repo_idx].repo;
                                        if let Some(ref branches) = repo_items[*repo_idx].branches {
                                            if let Some((branch, _)) = branches.get(*branch_idx) {
                                                let url = format!("{}/tree/{}", repo.html_url, branch.name);
                                                let _ = open::that(url);
                                            }
                                        }
                                    }
                                    DisplayItem::Commit { repo_idx, branch_idx, commit_idx } => {
                                        let repo = &repo_items[*repo_idx].repo;
                                        if let Some(ref branches) = repo_items[*repo_idx].branches {
                                            if let Some((_, commits)) = branches.get(*branch_idx) {
                                                if let Some(commit) = commits.get(*commit_idx) {
                                                    let url = format!("{}/commit/{}", repo.html_url, commit.sha);
                                                    let _ = open::that(url);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn fetch_branches_and_commits(
    client: &Client,
    token: &str,
    owner: &str,
    repo_name: &str,
) -> Vec<(Branch, Vec<CommitWrapper>)> {
    let branches_url = format!("https://api.github.com/repos/{}/{}/branches", owner, repo_name);

    let res = client
        .get(&branches_url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    let mut result = Vec::new();

    if let Ok(response) = res {
        if response.status().is_success() {
            let branches: Vec<Branch> = response.json().unwrap_or_default();
            for branch in branches {
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

                result.push((branch, commits));
            }
        }
    }

    result
}