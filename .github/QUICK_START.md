# CI/CD Quick Start Guide

## For Contributors

### Before Submitting a PR

Run these commands locally to catch issues before CI:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Build examples
cargo build --examples

# Check documentation
cargo doc --all-features --no-deps
```

### Understanding CI Checks

When you create a PR, these checks will run:

1. **Quick Checks** (~2 min)
   - Code formatting
   - Clippy lints
   - Build verification

2. **Test Suite** (~10 min)
   - Tests on Ubuntu, macOS, Windows
   - Tests with stable and beta Rust
   - Integration tests

3. **Code Coverage** (~5 min)
   - Generates coverage report
   - Uploads to Codecov

4. **Examples** (~3 min)
   - Verifies all examples compile

5. **Security Audit** (~2 min)
   - Checks for vulnerabilities
   - Validates licenses

6. **MSRV Check** (~3 min)
   - Verifies Rust 1.75.0 compatibility

7. **Documentation** (~2 min)
   - Builds docs without warnings

All checks must pass before merging.

## For Maintainers

### Creating a Release

1. **Update version in Cargo.toml**
   ```toml
   [package]
   version = "0.2.0"  # Update this
   ```

2. **Update CHANGELOG.md**
   ```markdown
   ## [0.2.0] - 2024-10-24
   ### Added
   - New feature X
   ### Fixed
   - Bug Y
   ```

3. **Commit changes**
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

4. **Create and push tag**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

5. **Monitor release workflow**
   - Go to GitHub Actions tab
   - Watch the release workflow
   - Verify:
     - ✅ GitHub release created
     - ✅ Published to crates.io
     - ✅ Documentation deployed

### Required Secrets

Set these in: Repository Settings → Secrets and variables → Actions

1. **CARGO_REGISTRY_TOKEN**
   - Get from: https://crates.io/me
   - Create new token with "publish" scope
   - Add as repository secret

2. **CODECOV_TOKEN** (optional)
   - Sign up at: https://codecov.io
   - Add repository
   - Copy token
   - Add as repository secret

### Enabling GitHub Pages

1. Go to: Repository Settings → Pages
2. Source: Deploy from a branch
3. Branch: gh-pages
4. Folder: / (root)
5. Save

Documentation will be available at:
`https://ticktockbent.github.io/Taskline/`

### Managing Dependabot

Dependabot will create PRs for:
- Cargo dependency updates (weekly, Monday 6 AM)
- GitHub Actions updates (weekly, Monday 6 AM)

To merge a dependabot PR:
1. Review the changelog of the updated dependency
2. Check CI passes
3. Merge if everything looks good

### Security Advisories

Weekly security scans run automatically.

If vulnerabilities are found:
1. Check the GitHub Security tab
2. Review the advisory
3. Update affected dependencies
4. Create a PR with fixes

## Troubleshooting

### "Format check failed"
```bash
cargo fmt --all
git add .
git commit -m "style: format code"
```

### "Clippy check failed"
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Fix the issues, then commit
```

### "Tests failed on Windows"
- Check for platform-specific issues
- Test locally on Windows if possible
- Consider using Windows-specific conditionals

### "MSRV check failed"
Either:
- Fix code to work with Rust 1.75.0, OR
- Update MSRV in README and workflow (if necessary)

### "Release failed: version mismatch"
- Ensure Cargo.toml version matches git tag
- Tag `v0.2.0` requires `version = "0.2.0"`

## Best Practices

✅ **DO:**
- Run tests locally before pushing
- Write descriptive commit messages
- Update documentation with code changes
- Add tests for new features
- Keep PRs focused and small

❌ **DON'T:**
- Force push to main
- Merge without CI passing
- Ignore clippy warnings
- Skip documentation updates
- Commit generated files

## Getting Help

- Check [WORKFLOWS.md](.github/WORKFLOWS.md) for detailed documentation
- Review failed CI logs in GitHub Actions
- Ask in issues or discussions

## Useful Commands

```bash
# Run all checks locally (what CI runs)
cargo fmt --all && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-features && \
cargo doc --all-features --no-deps

# Check specific example
cargo check --example basic_scheduler

# Generate documentation locally
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Audit for security vulnerabilities
cargo audit
```
