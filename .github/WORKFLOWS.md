# CI/CD Workflows Documentation

This document describes the GitHub Actions workflows configured for the Taskline project.

## Workflows Overview

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches
- Weekly security audit (Sunday at midnight)

**Jobs:**

#### Quick Checks (runs first, fail-fast)
- **Format Check**: Ensures code follows rustfmt standards
- **Clippy**: Lints code with all warnings as errors
- **Build Check**: Verifies compilation

#### Test Suite (matrix strategy)
- **Platforms**: Ubuntu, macOS, Windows
- **Rust Versions**: stable, beta, nightly (nightly only on Ubuntu)
- **Tests**:
  - All features enabled
  - No default features
  - Doc tests
  - Integration tests

#### Code Coverage
- Generates coverage report using cargo-tarpaulin
- Uploads to Codecov
- Stores coverage artifact

#### Build Release
- Tests release builds on all platforms

#### Examples
- Builds all example files
- Verifies each example compiles

#### Security Audit
- Runs `cargo audit` for known vulnerabilities
- Runs `cargo-deny` to check advisories

#### MSRV Check
- Verifies compatibility with Rust 1.75.0 (minimum supported version)

#### Documentation
- Builds documentation with warnings as errors
- Uploads documentation artifacts

#### CI Success
- Final job that verifies all required checks passed
- Provides clear success/failure status

### 2. Release Workflow (`.github/workflows/release.yml`)

**Triggers:**
- Push tags matching `v*.*.*` (e.g., v0.1.0, v1.2.3)

**Jobs:**

#### Pre-Release Verification
- Verifies version in Cargo.toml matches the git tag
- Runs full test suite
- Runs doc tests
- Builds release binary
- Checks documentation builds without warnings

#### Create GitHub Release
- Extracts version from tag
- Creates GitHub release with auto-generated release notes

#### Publish to crates.io
- Publishes package to crates.io
- Requires `CARGO_REGISTRY_TOKEN` secret

#### Deploy Documentation
- Builds documentation
- Deploys to GitHub Pages

#### Announce Release
- Logs successful release with all deployment statuses

## Configuration Files

### `deny.toml` - Cargo Deny Configuration

Security and license policy:
- **Advisories**: Denies security vulnerabilities
- **Licenses**: Allows only approved open-source licenses (MIT, Apache-2.0, BSD, etc.)
- **Bans**: Warns about multiple versions of dependencies
- **Sources**: Only allows crates from crates.io

### `.github/dependabot.yml` - Automated Dependency Updates

- **Cargo dependencies**: Checked weekly on Monday at 6 AM
- **GitHub Actions**: Checked weekly on Monday at 6 AM
- Auto-assigns PRs to maintainers
- Labels PRs appropriately

## Secrets Required

The following GitHub secrets must be configured:

1. **`CARGO_REGISTRY_TOKEN`** - Token for publishing to crates.io
   - Get from: https://crates.io/me
   - Required for: Release workflow

2. **`CODECOV_TOKEN`** (optional) - Token for Codecov uploads
   - Get from: https://codecov.io
   - Required for: Code coverage workflow

3. **`GITHUB_TOKEN`** - Automatically provided by GitHub
   - No configuration needed

## Branch Protection Rules (Recommended)

For the `main` branch:

1. **Require pull request reviews before merging**
   - Required approving reviews: 1

2. **Require status checks to pass before merging**
   - Required checks:
     - `Quick Checks`
     - `Test Suite (ubuntu-latest, stable)`
     - `MSRV Check`
     - `Documentation`
     - `Examples`
     - `Security Audit`

3. **Require branches to be up to date before merging**

4. **Do not allow force pushes**

5. **Require linear history** (optional but recommended)

## Performance Optimizations

1. **Rust Cache**: Uses `Swatinem/rust-cache@v2` for fast dependency caching
2. **Fail-Fast**: Quick checks run first to catch common issues early
3. **Job Dependencies**: Jobs run in parallel where possible
4. **Matrix Strategy**: Tests run concurrently across platforms/versions

## Maintenance

### Weekly Tasks
- Dependabot automatically creates PRs for dependency updates
- Security audit runs automatically

### When Adding New Examples
Examples are automatically discovered - no workflow changes needed.

### When Updating MSRV
Update the Rust version in the MSRV check job (currently 1.75.0).

### When Adding New Features
Add appropriate feature flags to test commands if needed.

## Troubleshooting

### Failed CI Checks

1. **Format Check Failed**
   ```bash
   cargo fmt --all
   ```

2. **Clippy Failed**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Tests Failed**
   ```bash
   cargo test --all-features --verbose
   ```

4. **MSRV Check Failed**
   - Update minimum Rust version in README
   - Or fix code to work with older Rust versions

### Failed Release

1. **Version Mismatch**
   - Ensure Cargo.toml version matches git tag
   - Example: Tag `v0.1.0` requires `version = "0.1.0"` in Cargo.toml

2. **crates.io Publish Failed**
   - Verify `CARGO_REGISTRY_TOKEN` is set correctly
   - Ensure version doesn't already exist on crates.io

3. **Documentation Deploy Failed**
   - Check that GitHub Pages is enabled in repository settings
   - Verify workflow has `pages: write` permission

## Best Practices

1. **Always run tests locally before pushing**
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features
   cargo fmt --all -- --check
   ```

2. **For releases:**
   - Update CHANGELOG.md
   - Bump version in Cargo.toml
   - Create annotated tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
   - Push tag: `git push origin v0.1.0`

3. **Keep dependencies updated**
   - Review and merge dependabot PRs regularly
   - Test after updating major dependencies

## Metrics and Monitoring

- **Code Coverage**: View at Codecov dashboard
- **Build Times**: Check GitHub Actions tab
- **Security Advisories**: Monitored via cargo-audit and cargo-deny
- **Documentation**: Auto-deployed to GitHub Pages after releases

## Future Enhancements

Potential improvements for the CI/CD pipeline:

- [ ] Add performance benchmarking (when benchmarks are re-added)
- [ ] Integration testing with real cron jobs
- [ ] Docker image publishing
- [ ] Automated changelog generation
- [ ] Release candidate (RC) workflow
- [ ] Canary deployments
- [ ] A/B testing framework integration
