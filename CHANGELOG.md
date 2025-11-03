# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-11-03

### Changed
- **BREAKING**: Renamed package from `cronline` to `cronline` (crates.io name collision)
- Updated dependency: `cron` from 0.12.0 to 0.15.0
- Updated dependency: `rand` from 0.8.5 to 0.9.2 (dev-dependency)
- Updated dependency: `thiserror` from 1.0.56 to 2.0.17
- Fixed `rand::random::<usize>()` compatibility for rand 0.9 (now uses `u32` and casts to `usize`)

### Infrastructure
- Updated GitHub Actions: `actions/checkout` from v4 to v5
- Updated GitHub Actions: `actions/upload-artifact` from v4 to v5
- Updated GitHub Actions: `peaceiris/actions-gh-pages` from v3 to v4
- Updated GitHub Actions: `softprops/action-gh-release` from v1 to v2
- Fixed MSRV check to use `dtolnay/rust-toolchain@master` with explicit `toolchain: "1.75.0"`

## [0.2.0] - 2024-10-24

### Added

#### Core Features
- **Interval-based scheduling**: Tasks can now be scheduled using fixed time intervals with `Task::with_interval(Duration)`
- **Task tags/labels**: Add tags to tasks for grouping and filtering using `with_tags()`, `with_tag()`, and `has_tag()`
- **Graceful task cancellation**: Tasks can be cancelled gracefully using cancellation tokens
- **Task timeout warnings**: Tasks now emit warnings at 80% of timeout threshold
- **Event bus system**: Comprehensive event system for scheduler and task lifecycle events
  - Events: `SchedulerStarted`, `SchedulerStopped`, `TaskAdded`, `TaskRemoved`, `TaskStarting`, `TaskCompleted`, `TaskFailed`, `TaskTimedOut`, `TaskTimeoutWarning`, `TaskCancelled`, `TaskStatusChanged`, `TaskPaused`, `TaskResumed`
- **Auto-generated task names**: Tasks automatically get descriptive names based on cron expressions or intervals
- **Tag filtering methods**: Filter tasks by tags using `tasks_with_tag()`, `tasks_with_any_tag()`, `tasks_with_all_tags()`

#### New Types
- `ScheduleType` enum: Supports both `Cron` and `Interval` scheduling
- `EventBus`: Broadcast-based event system for scheduler events
- `SchedulerEvent` enum: Comprehensive event types for all scheduler operations

#### Examples
- `enhanced_features.rs`: Demonstration of all new features
- `web_integration_axum.rs`: Integration with Axum web framework
- `monitoring_alerting.rs`: System monitoring and alerting patterns
- `database_maintenance.rs`: Database maintenance task patterns
- `api_polling_scraping.rs`: API polling and rate-limited scraping
- `notification_system.rs`: Multi-channel notification system
- `cache_warming.rs`: Cache management and warming strategies

### Changed

#### Breaking Changes
- `Scheduler::add()` is now async and returns `Future<Output = Result<String>>`
- `Scheduler::add_task()` is now async and returns `Future<Output = Result<String>>`
- `Scheduler::remove()` is now async and returns `Future<Output = Result<()>>`
- `ScheduleType::Cron` now boxes `CronSchedule` to reduce enum size: `Cron(Box<CronSchedule>)`

#### Improvements
- Implemented `Default` trait for `Scheduler`
- Simplified `Option::map` patterns in cron_parser and scheduler
- Optimized enum variant sizes for better memory usage
- Added comprehensive documentation to all public APIs

### Fixed
- Cron expressions now automatically convert 5-field format to 6-field (with seconds)
- Fixed timeout detection logic using `ExecutionOutcome` enum
- Fixed `should_execute_at()` to use `after()` instead of deprecated `before()` method
- Corrected Cargo.lock version compatibility (version 3)

### Infrastructure

#### CI/CD Improvements
- Consolidated GitHub Actions workflows from 5 files to 2 comprehensive workflows
- Added multi-platform testing (Ubuntu, macOS, Windows)
- Added multi-version testing (stable, beta, nightly)
- Integrated code coverage reporting with Codecov
- Added security auditing with cargo-audit and cargo-deny
- Added MSRV (Minimum Supported Rust Version) checks for Rust 1.75.0
- Implemented fail-fast strategy with quick checks
- Modern caching with `Swatinem/rust-cache@v2`
- Weekly scheduled security scans

#### CI/CD Files
- Revamped `.github/workflows/ci.yml` with 8 job types
- Enhanced `.github/workflows/release.yml` for automated releases
- Added `deny.toml` for cargo-deny security configuration
- Added `.github/dependabot.yml` for automated dependency updates
- Added PR and issue templates for better contribution workflow
- Created comprehensive CI/CD documentation in `.github/WORKFLOWS.md`
- Added quick reference guide in `.github/QUICK_START.md`

#### Code Quality
- Fixed all rustfmt formatting issues (2000+ lines formatted)
- Fixed all clippy warnings and errors
- Added lint suppressions for example code
- Ensured compatibility with `--no-default-features` builds
- Made `env_logger` optional dependency

#### Documentation
- Added `EXAMPLES.md` with comprehensive example documentation
- Created `CI_FIXES.md` documenting all CI/CD improvements
- Added `TEST_RESULTS.md` with detailed test coverage information

### Dependencies
- Updated to use Rust 1.90.0 stable toolchain
- Added `tokio-util = "0.7.10"` for cancellation tokens
- Made `env_logger` optional with `basic-logging` feature flag

### Testing
- All 44 library unit tests passing
- All 11 integration tests passing
- All 50 doc tests passing
- Examples compile with and without default features
- 100% pass rate across all test suites

## [0.1.0] - Initial Release

### Added
- Basic task scheduling with cron expressions
- Asynchronous task execution using Tokio
- Task configuration (timeout, retries, error handling)
- Task statistics tracking
- Task pause/resume functionality
- Scheduler lifecycle management
- Multiple scheduler support
- Task status tracking (Idle, Running, Paused, Completed, Failed)

[0.2.1]: https://github.com/TickTockBent/Cronline/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/TickTockBent/Cronline/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/TickTockBent/Cronline/releases/tag/v0.1.0
