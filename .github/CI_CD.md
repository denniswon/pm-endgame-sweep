# CI/CD Pipeline Documentation

## Overview

This repository uses a comprehensive, security-hardened CI/CD pipeline with GitHub Actions to ensure code quality, security, and reliability before merging to `main`.

## Workflow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     PR Opened/Updated                        │
└──────────────────┬──────────────────────────────────────────┘
                   │
        ┌──────────┴──────────┐
        │   pr-checks.yml     │
        │  (Orchestrator)     │
        └──────────┬──────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
┌───▼────┐   ┌────▼────┐   ┌────▼────┐
│ Detect │   │  Meta   │   │Security │
│Changes │   │  Check  │   │ Review  │
└───┬────┘   └─────────┘   └─────────┘
    │
    ├─[Rust Changed]──────┐
    │                     │
    │            ┌────────▼────────┐
    │            │   rust-ci.yml   │
    │            │                 │
    │            │ • Lint (fmt)    │
    │            │ • Clippy        │
    │            │ • Tests         │
    │            │ • Build         │
    │            └─────────────────┘
    │
    └─[Web Changed]───────┐
                          │
                  ┌───────▼────────┐
                  │  web-ci.yml    │
                  │                │
                  │ • ESLint       │
                  │ • TypeCheck    │
                  │ • Unit Tests   │
                  │ • Build        │
                  └────────────────┘
```

## Workflows

### 1. PR Checks (`pr-checks.yml`)

**Purpose**: Orchestrator workflow that runs all required pre-merge checks.

**Triggers**:

- Pull requests to `main`
- PR synchronize, reopen, ready for review

**Features**:

- ✅ **Smart Detection**: Only runs relevant checks based on file changes
- ✅ **Concurrency Control**: Cancels outdated runs automatically
- ✅ **Dependency Review**: Scans for vulnerable dependencies
- ✅ **PR Metadata**: Validates title format and size

**Jobs**:

1. **pr-metadata**

   - Validates PR title (conventional commits format)
   - Warns on large PRs (>50 files or >1000 lines)
   - Timeout: 5 minutes

2. **dependency-review**

   - Scans for security vulnerabilities
   - Blocks moderate+ severity issues
   - Validates licenses (MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, 0BSD, Zlib, CC0-1.0, BlueOak-1.0.0, CDLA-Permissive-2.0, Unicode-3.0, Unlicense)
   - Timeout: 5 minutes

3. **detect-changes**

   - Uses `dorny/paths-filter` for smart detection
   - Outputs: `rust`, `web`, `docs`
   - Timeout: 5 minutes

4. **rust-checks** (conditional)

   - Runs only if Rust files changed
   - Formatting check (`cargo fmt`)
   - Linting (`cargo clippy -D warnings`)
   - Database migrations
   - Full test suite
   - Timeout: 30 minutes

5. **web-checks** (conditional)

   - Runs only if web files changed
   - ESLint validation
   - TypeScript type checking
   - Unit tests
   - Production build
   - Timeout: 20 minutes

6. **pr-status**
   - Final status aggregator
   - Fails if any required check fails
   - Always runs (even if jobs skipped)

**Required Checks**:

- `pr-metadata`: ✅ Required
- `dependency-review`: ✅ Required
- `rust-checks`: ✅ Required (if Rust files changed)
- `web-checks`: ✅ Required (if web files changed)
- `pr-status`: ✅ Required

### 2. Rust CI (`rust-ci.yml`)

**Purpose**: Comprehensive Rust backend testing and validation.

**Triggers**:

- Push to `main`, `dev`
- Pull requests to `main`
- Only when Rust files change

**Features**:

- ✅ **Fast Caching**: Uses `Swatinem/rust-cache` with shared keys
- ✅ **PostgreSQL Service**: In-container database for tests
- ✅ **Multi-OS Support**: Ready for macOS/Windows (currently Ubuntu only)
- ✅ **Security Audit**: cargo-audit for vulnerability scanning

**Jobs**:

1. **lint** (Priority: High, runs first)

   - `cargo fmt --check`
   - `cargo clippy --all-targets --all-features -D warnings`
   - Cache: Shared across jobs
   - Timeout: 10 minutes

2. **security** (Parallel)

   - `cargo audit --deny warnings`
   - Continue on error (informational)
   - Cached binary
   - Timeout: 10 minutes

3. **test**

   - PostgreSQL 16 service container
   - SQLx migrations
   - `cargo build --workspace --all-targets`
   - `cargo test --workspace`
   - `cargo test --doc`
   - Timeout: 30 minutes

4. **build-release**

   - Release mode compilation
   - Binary size reporting
   - Timeout: 20 minutes

5. **coverage** (main branch only)
   - Uses `cargo-llvm-cov`
   - Uploads to Codecov
   - Timeout: 30 minutes

**Caching Strategy**:

```yaml
Cache Key: rust-{job}-{os}-{Cargo.lock hash}
Cached:
  - ~/.cargo/registry/index
  - ~/.cargo/registry/cache
  - ~/.cargo/git/db
  - target/
Invalidation: Cargo.lock changes
```

### 3. Web CI (`web-ci.yml`)

**Purpose**: Frontend testing, linting, and build validation.

**Triggers**:

- Push to `main`, `dev`
- Pull requests to `main`
- Only when web files change

**Features**:

- ✅ **Yarn Caching**: Built-in Node.js action caching
- ✅ **Concurrency Control**: Auto-cancel old runs
- ✅ **Coverage Upload**: Codecov integration
- ✅ **E2E on Main**: Playwright tests only on main branch

**Jobs**:

1. **lint** (Priority: High)

   - `yarn lint` (ESLint)
   - `yarn tsc --noEmit` (TypeScript)
   - Timeout: 10 minutes

2. **test**

   - `yarn test --run` (Vitest)
   - `yarn test:coverage`
   - Codecov upload
   - Timeout: 15 minutes

3. **build**

   - `yarn build` (Next.js)
   - Build output analysis
   - Timeout: 15 minutes

4. **e2e** (main branch only)
   - Playwright with Chromium
   - Artifact upload on failure
   - Timeout: 20 minutes

**Caching Strategy**:

```yaml
Cache Key: yarn-{os}-{yarn.lock hash}
Cached:
  - ~/.yarn/cache
  - node_modules
Invalidation: yarn.lock changes
```

### 4. Security Scan (`security-scan.yml`)

**Purpose**: Automated security scanning and vulnerability detection.

**Triggers**:

- Daily at 6 AM UTC
- Manual dispatch
- PRs modifying dependencies

**Jobs**:

1. **rust-audit**

   - `cargo audit --deny warnings`
   - Uploads failure reports
   - Timeout: 10 minutes

2. **npm-audit**

   - `yarn audit --level moderate`
   - JSON report generation
   - Timeout: 10 minutes

3. **codeql**

   - GitHub CodeQL analysis
   - JavaScript/TypeScript scanning
   - Security and quality queries
   - Timeout: 30 minutes

4. **secret-scan** (PRs only)
   - TruffleHog OSS
   - Verified secrets only
   - Diff-based scanning

## Automation Features

### Dependabot (`dependabot.yml`)

**Automated Dependency Updates**:

**Cargo (Rust)**:

- Weekly scans (Monday 9 AM)
- Groups minor/patch updates
- Max 5 open PRs
- Auto-labels: `dependencies`, `rust`

**NPM (Web)**:

- Weekly scans (Monday 9 AM)
- Ignores major version updates
- Groups minor/patch updates
- Max 5 open PRs
- Auto-labels: `dependencies`, `web`

**GitHub Actions**:

- Weekly scans (Monday 9 AM)
- Max 3 open PRs
- Auto-labels: `dependencies`, `ci`

### CODEOWNERS

Automatic review requests for:

- Rust: `@denniswon`
- Web: `@denniswon`
- CI/CD: `@denniswon`
- Migrations: `@denniswon` (critical)

### PR Template

Standardized PR format with:

- Description and change type
- Testing checklist
- Code quality checks
- Security review
- Documentation updates
- Deployment notes

## Performance Optimizations

### 1. Caching Strategy

**Rust**:

```yaml
Tool: Swatinem/rust-cache@v2
Benefits:
  - Incremental compilation disabled (better CI cache hits)
  - Shared cache keys across jobs
  - Cache even on failure (save-if: true)
  - Automatic cache pruning
Speed Improvement: 5-10x faster rebuilds
```

**Node.js**:

```yaml
Tool: actions/setup-node@v4 with yarn cache
Benefits:
  - Native Yarn v1 caching
  - Lockfile-based invalidation
  - Offline mode for reliability
Speed Improvement: 3-5x faster installs
```

**Binary Tools**:

```yaml
Cached:
  - cargo-audit
  - sqlx-cli
  - cargo-llvm-cov
Invalidation: Manual (versioned keys)
```

### 2. Concurrency Control

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Benefits**:

- Cancels outdated PR runs automatically
- Saves compute resources
- Faster feedback on latest commit

### 3. Job Ordering

**Fast Fail Strategy**:

1. Lint jobs (1-2 min) - fail fast on style issues
2. Type checking (2-3 min)
3. Tests (5-15 min)
4. Builds (10-20 min)

### 4. Smart Triggering

**Path Filters**:

```yaml
paths:
  - "crates/**" # Rust changes
  - "web/**" # Web changes
  - "Cargo.lock" # Dependency changes
```

**Benefits**:

- Skip irrelevant workflows
- Reduce queue time
- Lower compute costs

## Security Features

### 1. Dependency Scanning

**Multi-Layer Defense**:

- ✅ Dependabot: Automated updates
- ✅ Dependency Review: PR-time blocking
- ✅ cargo-audit: Rust vulnerability DB
- ✅ yarn audit: NPM vulnerability DB

### 2. Code Scanning

**CodeQL**:

- Security queries: SQL injection, XSS, etc.
- Quality queries: Code smells, bugs
- JavaScript/TypeScript coverage

**TruffleHog**:

- Secret detection in diffs
- Verified secrets only (reduce false positives)

### 3. License Compliance

**Allowed Licenses**:

- MIT
- Apache-2.0
- BSD-3-Clause
- ISC
- 0BSD

**Blocked**: Copyleft licenses (GPL, AGPL) by default

### 4. Permissions

**Principle of Least Privilege**:

```yaml
permissions:
  contents: read # Most jobs
  security-events: write # CodeQL only
```

## Branch Protection Rules

### Recommended Settings for `main`

```yaml
Required Status Checks: ✅ PR Checks / pr-status
  ✅ Rust CI / lint
  ✅ Rust CI / test
  ✅ Web CI / lint
  ✅ Web CI / test
  ✅ Web CI / build

Required Reviews: ✅ 1 approving review
  ✅ Dismiss stale reviews on push
  ✅ Require review from code owners

Branch Rules: ✅ Require branches to be up to date
  ✅ Require conversation resolution
  ✅ No force pushes
  ✅ No deletions

Additional: ✅ Require signed commits (recommended)
  ✅ Require linear history (optional)
```

### Setup Instructions

1. **Go to Repository Settings → Branches**
2. **Add rule for `main`**
3. **Enable**:
   - Require pull request before merging
   - Require status checks to pass
   - Select all required checks above
   - Require branches to be up to date
   - Require conversation resolution before merging

## Monitoring & Debugging

### Workflow Status

View all runs:

```
https://github.com/denniswon/pm-endgame-sweep/actions
```

### Failed Run Debugging

1. **Check Logs**:

   - Click on failed job
   - Expand failing step
   - Look for error messages

2. **Artifacts**:

   - Playwright reports (E2E failures)
   - Audit reports (security failures)
   - Coverage reports

3. **Re-run Jobs**:
   - Re-run failed jobs only
   - Re-run all jobs

### Common Issues

**Issue**: Rust cache misses
**Solution**: Check Cargo.lock unchanged, verify cache key

**Issue**: Web build fails on CI but works locally
**Solution**: Check Node.js version match (22), clear cache

**Issue**: Tests timeout
**Solution**: Increase timeout, check database connectivity

**Issue**: Dependabot PRs fail
**Solution**: Review breaking changes, update code accordingly

## Cost Optimization

### GitHub Actions Minutes Usage

**Free Tier** (Public repos):

- Unlimited minutes for public repositories
- ✅ All current workflows included

**Estimated Usage** (per PR):

```
Rust CI:     ~15 minutes
Web CI:      ~10 minutes
PR Checks:   ~30 minutes
Total:       ~55 minutes per PR
```

**Monthly** (10 PRs):

```
Total: ~550 minutes
Cost: $0 (public repo)
```

### Optimization Tips

1. **Skip E2E on PRs** (already implemented)
2. **Cache aggressively** (already implemented)
3. **Use path filters** (already implemented)
4. **Cancel stale runs** (already implemented)

## Metrics & KPIs

### Target Metrics

```
✅ Lint feedback:     < 2 minutes
✅ Test feedback:     < 15 minutes
✅ Full pipeline:     < 30 minutes
✅ Cache hit rate:    > 80%
✅ Flaky test rate:   < 1%
✅ Security issues:   0 critical/high
```

### Current Performance

```
Rust lint:     ~2 minutes
Rust test:     ~10 minutes
Web lint:      ~1 minute
Web test:      ~3 minutes
Web build:     ~5 minutes
Total PR:      ~25 minutes ✅
```

## Migration Guide

### From Previous Setup

If migrating from existing CI:

1. **Backup existing workflows**
2. **Remove old workflow files**
3. **Add new workflows** (already done)
4. **Update branch protection rules**
5. **Test with draft PR**
6. **Monitor first few runs**

### Gradual Rollout

1. **Week 1**: Enable on feature branches
2. **Week 2**: Enable on `main` with optional checks
3. **Week 3**: Make checks required
4. **Week 4**: Enable all security features

## Support

### Documentation

- [GitHub Actions Docs](https://docs.github.com/actions)
- [Rust Cache Action](https://github.com/Swatinem/rust-cache)
- [Dependabot Docs](https://docs.github.com/code-security/dependabot)

### Troubleshooting

For CI/CD issues:

1. Check workflow logs
2. Review recent changes
3. Test locally first
4. Open issue with logs

---

**Last Updated**: 2026-01-03
**Version**: 1.0
**Maintained by**: @denniswon
