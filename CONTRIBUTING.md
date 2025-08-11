# Contributing

## Branch Conventions

**Name your branches correctly:**
- Features: `features/feature-name`
- Fixes: `fix/bug-name`
- Use `-` for word separation

## Development Process

**Merge Schedule:**
- PRs merged into `develop` by Thursday night (every two weeks)
- `develop` merged into `main` on Friday before meetings
- Version bumps: increment second number (e.g., v0.1.0 → v0.2.0)

**After merging: delete the branch**

## Definition of Done

- **Self-contained PRs** - no half-implemented features
- **Code Review:**
  - Features: 2 reviewers
  - Bugfixes: 1 reviewer
- **CI Requirements:**
  - Code compiles on latest Rust
  - `rustfmt` passes
  - `clippy` passes
  - Some warnings treated as errors
- **Tests:** Basic unit tests required
- **PR Description:** Brief description mandatory
- **Documentation:**
  - Document non-trivial functions
  - Document user-facing changes

## Code Style

- Comments start with space + uppercase letter
- Use `tracing` for logging (`info!`, `warn!`, `error!`, `debug!`)
- Follow existing patterns in codebase

## Getting Started

1. Fork the repo
2. Create your branch: `git checkout -b features/your-feature`
3. Make changes and test: `cargo test`
4. Ensure CI passes: `cargo fmt && cargo clippy`
5. Create PR with clear description
