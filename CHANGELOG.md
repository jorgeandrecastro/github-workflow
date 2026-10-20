### 📄 `CHANGELOG.md`

```markdown
# Changelog

All notable changes to the **github-workflow** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-02-23

### Added
- **Asynchronous Data Refresh (`r`)**: Added the ability to manually trigger a background refresh for any selected repository without blocking the UI.
- **Enhanced CI/CD State Detection**: Added support for intermediate `status` states (e.g., `in_progress` builds) alongside final `conclusion` statuses.
- **Improved Visual Theme**: Transitioned navigation highlight color to high-contrast green (`Theme::SUCCESS`) over anthracite background for superior contrast and readability.
- **Code Documentation**: Added Rust docstrings (`///`) across all structural data models in `src/models.rs`.

### Changed
- **API Pagination Limits**: Added `per_page=10` query parameter when fetching branch details to prevent hitting GitHub REST API rate limits under heavy multi-threading.
- **Codebase Standardization**: Standardized all internal comments, log messages, variable names, and UI elements to clean English.
- **UI Header Branding**: Updated dashboard header to reflect version `v0.2`.

### Fixed
- **Compiler Health**: Addressed all `dead_code` warnings regarding unused theme constants using explicit `#[allow(dead_code)]` annotations.
- **Thread Safety & Panic Prevention**: Replaced direct `.join().unwrap()` calls with `.filter_map(|h| h.join().ok())` during parallel scoped thread execution.
- **Safe JSON Deserialization**: Replaced potentially unsafe response unwrap patterns with typed `response.json::<T>()` handling.

