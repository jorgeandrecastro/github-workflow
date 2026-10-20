mod github;
mod models;
mod styles;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::Stylize;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, ListState, Paragraph},
};
use reqwest::blocking::Client;
use std::{
    env, io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use models::{BranchState, DisplayItem, RepoItem};
use styles::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = match env::var("GITHUB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Error: The GITHUB_TOKEN environment variable is not defined.");
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let repos = github::fetch_repos(&client, &token);

    let repo_items: Vec<RepoItem> = repos
        .into_iter()
        .map(|repo| RepoItem {
            repo,
            is_expanded: false,
            branches_state: BranchState::NotLoaded,
        })
        .collect();

    let repo_items = Arc::new(Mutex::new(repo_items));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    if !repo_items.lock().unwrap().is_empty() {
        list_state.select(Some(0));
    }

    // Text search management
    let mut search_mode = false;
    let mut search_query = String::new();

    loop {
        let mut display_map: Vec<(DisplayItem, usize)> = Vec::new();
        let items_guard = repo_items.lock().unwrap();

        // Dynamic repository filtering based on search query
        let filtered_indices: Vec<usize> = items_guard
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if search_query.is_empty() {
                    true
                } else {
                    item.repo
                        .name
                        .to_lowercase()
                        .contains(&search_query.to_lowercase())
                }
            })
            .map(|(idx, _)| idx)
            .collect();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Header
                        Constraint::Length(3), // Search bar
                        Constraint::Min(10),   // Main list
                        Constraint::Length(3), // Help / shortcuts bar
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Header v0.2
            let header = Paragraph::new(" 🚀 Github-workflow v0.2 — Dashboard ")
                .style(Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD))
                .block(Theme::block("Dashboard"));
            f.render_widget(header, chunks[0]);

            // Search bar
            let search_text = if search_mode {
                format!("🔍 Search: {}_", search_query)
            } else if search_query.is_empty() {
                "🔍 Press '/' to filter repositories...".to_string()
            } else {
                format!(
                    "🔍 Active filter: {} (Press Esc to clear)",
                    search_query
                )
            };
            let search_bar = Paragraph::new(search_text)
                .style(Style::default().fg(if search_mode {
                    Theme::WARNING
                } else {
                    Theme::SUBTLE
                }))
                .block(Theme::block("Search"));
            f.render_widget(search_bar, chunks[1]);

            let mut list_items: Vec<ListItem> = Vec::new();

            for &repo_idx in &filtered_indices {
                let item = &items_guard[repo_idx];
                display_map.push((DisplayItem::Repo { repo_idx }, repo_idx));

                let prefix = if item.is_expanded { "📂 [▼]" } else { "📁 [▶]" };
                let title = format!("{} {} ({})", prefix, item.repo.name, item.repo.html_url);

                let is_current_selected = list_state.selected() == Some(display_map.len() - 1);
                let style = if is_current_selected {
                    Theme::selected_item()
                } else {
                    Style::default().fg(Theme::PRIMARY)
                };

                list_items.push(ListItem::new(title).style(style));

                if item.is_expanded {
                    match &item.branches_state {
                        BranchState::NotLoaded | BranchState::Loading => {
                            display_map.push((DisplayItem::Loading, repo_idx));
                            list_items.push(
                                ListItem::new("     ⏳ Loading branches, PRs & CI...")
                                    .style(Style::default().fg(Theme::SUBTLE)),
                            );
                        }
                        BranchState::Loaded(branches) => {
                            if branches.is_empty() {
                                display_map.push((DisplayItem::Empty, repo_idx));
                                list_items.push(
                                    ListItem::new("     (No branches found)")
                                        .style(Style::default().fg(Theme::SUBTLE)),
                                );
                            } else {
                                for (branch_idx, b_info) in branches.iter().enumerate() {
                                    display_map.push((
                                        DisplayItem::Branch {
                                            repo_idx,
                                            branch_idx,
                                        },
                                        repo_idx,
                                    ));

                                    // CI/CD Status indicator
                                    let ci_icon = match b_info.ci_status.as_str() {
                                        "success" => "🟢".green(),
                                        "failure" => "🔴".red(),
                                        _ => "⚪".into(),
                                    };
                                    let is_b_selected =
                                        list_state.selected() == Some(display_map.len() - 1);
                                    let b_style = if is_b_selected {
                                        Theme::selected_item().add_modifier(Modifier::UNDERLINED)
                                    } else {
                                        Style::default().fg(Theme::ACCENT)
                                    };

                                    list_items.push(
                                        ListItem::new(format!(
                                            "     {} ├── 🌿 {}",
                                            ci_icon, b_info.branch.name
                                        ))
                                        .style(b_style),
                                    );

                                    // Pull Requests
                                    for (pr_idx, pr) in b_info.prs.iter().enumerate() {
                                        display_map.push((
                                            DisplayItem::PullRequest {
                                                repo_idx,
                                                branch_idx,
                                                pr_idx,
                                            },
                                            repo_idx,
                                        ));
                                        let is_pr_selected =
                                            list_state.selected() == Some(display_map.len() - 1);
                                        let pr_style = if is_pr_selected {
                                            Theme::selected_item()
                                        } else {
                                            Style::default().fg(Color::Rgb(175, 150, 220))
                                        };
                                        list_items.push(
                                            ListItem::new(format!("     │    🔀 PR: {}", pr.title))
                                                .style(pr_style),
                                        );
                                    }

                                    // Recent commits
                                    for (commit_idx, commit) in b_info.commits.iter().enumerate() {
                                        display_map.push((
                                            DisplayItem::Commit {
                                                repo_idx,
                                                branch_idx,
                                                commit_idx,
                                            },
                                            repo_idx,
                                        ));
                                        let short_sha = if commit.sha.len() >= 7 {
                                            &commit.sha[..7]
                                        } else {
                                            &commit.sha
                                        };
                                        let msg =
                                            commit.commit.message.lines().next().unwrap_or("");

                                        let is_c_selected =
                                            list_state.selected() == Some(display_map.len() - 1);
                                        let c_style = if is_c_selected {
                                            Theme::selected_item()
                                        } else {
                                            Style::default().fg(Theme::SUBTLE)
                                        };

                                        list_items.push(
                                            ListItem::new(format!(
                                                "     │    └── 💻 [{}] {}",
                                                short_sha, msg
                                            ))
                                            .style(c_style),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let list = List::new(list_items)
                .block(Theme::block("Repositories"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            f.render_stateful_widget(list, chunks[2], &mut list_state);

            // Help / Shortcuts (Updated to include 'r: Refresh')
            let help = Paragraph::new(
                " /: Search | r: Refresh | ↑/↓: Navigate | Space: Expand | o: Open Web | q: Quit ",
            )
            .style(Style::default().fg(Theme::SUBTLE))
            .block(Theme::block("Shortcuts"));
            f.render_widget(help, chunks[3]);
        })?;

        drop(items_guard);

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let mut items_guard = repo_items.lock().unwrap();

                if search_mode {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            search_mode = false;
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                        }
                        KeyCode::Char(c) => {
                            search_query.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            if !search_query.is_empty() {
                                search_query.clear();
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char('/') => {
                            search_mode = true;
                        }
                        // Added asynchronous refresh handler for 'r' key
                        KeyCode::Char('r') => {
                            if let Some(selected) = list_state.selected() {
                                if let Some((display_item, _)) = display_map.get(selected) {
                                    let repo_idx = match display_item {
                                        DisplayItem::Repo { repo_idx }
                                        | DisplayItem::Branch { repo_idx, .. }
                                        | DisplayItem::Commit { repo_idx, .. }
                                        | DisplayItem::PullRequest { repo_idx, .. } => *repo_idx,
                                        _ => usize::MAX,
                                    };

                                    if repo_idx < items_guard.len() {
                                        let item = &mut items_guard[repo_idx];
                                        item.branches_state = BranchState::Loading;

                                        let client_clone = client.clone();
                                        let token_clone = token.clone();
                                        let owner = item.repo.owner.login.clone();
                                        let repo_name = item.repo.name.clone();
                                        let repo_items_clone = Arc::clone(&repo_items);
                                        let target_idx = repo_idx;

                                        thread::spawn(move || {
                                            let branch_infos = github::fetch_branch_details(
                                                &client_clone,
                                                &token_clone,
                                                &owner,
                                                &repo_name,
                                            );
                                            let mut guard = repo_items_clone.lock().unwrap();
                                            guard[target_idx].branches_state =
                                                BranchState::Loaded(branch_infos);
                                        });
                                    }
                                }
                            }
                        }
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
                                if let Some((DisplayItem::Repo { repo_idx }, _)) =
                                    display_map.get(selected)
                                {
                                    let item = &mut items_guard[*repo_idx];
                                    item.is_expanded = !item.is_expanded;

                                    if item.is_expanded {
                                        if let BranchState::NotLoaded = item.branches_state {
                                            item.branches_state = BranchState::Loading;

                                            let client_clone = client.clone();
                                            let token_clone = token.clone();
                                            let owner = item.repo.owner.login.clone();
                                            let repo_name = item.repo.name.clone();
                                            let repo_items_clone = Arc::clone(&repo_items);
                                            let target_idx = *repo_idx;

                                            thread::spawn(move || {
                                                let branch_infos = github::fetch_branch_details(
                                                    &client_clone,
                                                    &token_clone,
                                                    &owner,
                                                    &repo_name,
                                                );
                                                let mut guard = repo_items_clone.lock().unwrap();
                                                guard[target_idx].branches_state =
                                                    BranchState::Loaded(branch_infos);
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('o') => {
                            if let Some(selected) = list_state.selected() {
                                if let Some((display_item, _)) = display_map.get(selected) {
                                    match display_item {
                                        DisplayItem::Repo { repo_idx } => {
                                            let url = &items_guard[*repo_idx].repo.html_url;
                                            let _ = open::that(url);
                                        }
                                        DisplayItem::Branch {
                                            repo_idx,
                                            branch_idx,
                                        } => {
                                            let repo = &items_guard[*repo_idx].repo;
                                            if let BranchState::Loaded(ref branches) =
                                                items_guard[*repo_idx].branches_state
                                            {
                                                if let Some(b_info) = branches.get(*branch_idx) {
                                                    let url = format!(
                                                        "{}/tree/{}",
                                                        repo.html_url, b_info.branch.name
                                                    );
                                                    let _ = open::that(url);
                                                }
                                            }
                                        }
                                        DisplayItem::PullRequest {
                                            repo_idx,
                                            branch_idx,
                                            pr_idx,
                                        } => {
                                            if let BranchState::Loaded(ref branches) =
                                                items_guard[*repo_idx].branches_state
                                            {
                                                if let Some(b_info) = branches.get(*branch_idx) {
                                                    if let Some(pr) = b_info.prs.get(*pr_idx) {
                                                        let _ = open::that(&pr.html_url);
                                                    }
                                                }
                                            }
                                        }
                                        DisplayItem::Commit {
                                            repo_idx,
                                            branch_idx,
                                            commit_idx,
                                        } => {
                                            let repo = &items_guard[*repo_idx].repo;
                                            if let BranchState::Loaded(ref branches) =
                                                items_guard[*repo_idx].branches_state
                                            {
                                                if let Some(b_info) = branches.get(*branch_idx) {
                                                    if let Some(commit) =
                                                        b_info.commits.get(*commit_idx)
                                                    {
                                                        let url = format!(
                                                            "{}/commit/{}",
                                                            repo.html_url, commit.sha
                                                        );
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
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}