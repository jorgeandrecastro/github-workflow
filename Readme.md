<div align="center">

# github-workflow 🚀

**Browse your GitHub repositories, branches, pull requests, and CI/CD statuses — right from your terminal.**

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Ratatui](https://img.shields.io/badge/TUI-Ratatui-blue?style=for-the-badge)](https://ratatui.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](https://opensource.org/licenses/MIT)

<br>
<img src="assets/image1.png" alt="github-workflow demo" width="800"/>
<br>
<img src="assets/image2.png" alt="github-workflow demo" width="800"/>


</div>

---

## Why github-workflow?

Switching between your terminal, browser, and GitHub just to check a pull request or CI run can interrupt your flow. **github-workflow** is a fast, asynchronous terminal UI that brings your GitHub activity into one place—without leaving the command line.

## Features

| | |
| --- | --- |
| ⚡ **Responsive interface** | Rust-powered background workers keep the UI responsive during API calls. |
| 📂 **On-demand loading** | Repositories load first; branches, commits, and pull requests are fetched as you explore. |
| 🔎 **Live filtering** | Press `/` to filter repositories by name. |
| ✅ **CI/CD status** | Check status indicators appear alongside branches. |
| 🔀 **Pull request overview** | See open pull requests associated with your branches. |
| 🌐 **Open in browser** | Press `o` on a repository, branch, pull request, or commit to open it on GitHub. |

## Getting started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- A GitHub Personal Access Token (PAT) with the required repository read access

### Install and run

1. Clone the repository:

   ```bash
   git clone https://github.com/jorgeandrecastro/github-workflow.git
   cd github-workflow
   ```

2. Set your GitHub token:

   ```bash
   export GITHUB_TOKEN="your_personal_access_token_here"
   ```

3. Build and launch the app:

   ```bash
   cargo run --release
   ```

## Keyboard shortcuts

| Key | Action |
| --- | --- |
| `↑` / `↓` | Navigate repositories, branches, and commits |
| `Space` / `Enter` | Expand or collapse the repository tree |
| `/` | Filter repositories by name |
| `o` | Open the selected item in your browser |
| `q` / `Esc` | Quit the application |

## Tech stack

| Component | Technology |
| --- | --- |
| Language | Rust |
| TUI framework | [Ratatui](https://ratatui.rs/) and Crossterm |
| API client | Reqwest and Serde |
| Browser integration | Open |

## Contributing

Contributions, issues, and feature requests are welcome. Feel free to check out the [open issues](https://github.com/jorgeandrecastro/github-workflow/issues).

## License

Distributed under the MIT License.
