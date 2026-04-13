# Contributing to mlbt

Thanks for your interest in contributing! A few guidelines to keep things
smooth.

## Issues

GitHub issues are the best place for bug reports, feature requests, and ideas.
Before opening a new issue, search existing ones to see if it's already been
raised. When filing a bug, include reproduction steps and your terminal/OS if
relevant.

## Branches

- Create a feature branch off `main` for your changes. Don't commit directly to
  your fork's `main`.
- Keep PRs scoped to a single feature or fix. Smaller PRs are easier to review
  and merge.
- For larger features, please open an issue first to discuss the approach
  before starting work. This avoids surprises and saves you from having to
  rework large changes during review.

## Code style

Before pushing, run both of these and make sure they're clean:

```sh
cargo fmt
cargo clippy --workspace -- --deny warnings
```

Both must pass with zero warnings. CI will fail your PR if either `cargo fmt`
or `cargo clippy` reports issues, so please run them locally before pushing.

## Conventions

- Look for existing patterns before introducing new ones. Shared helpers (color
  utilities, display traits, etc.) live in `src/components/util.rs`.
- Match the style of the surrounding code.

## API crate

The `mlbt-api` crate is a separate crate (`api/`) that wraps the MLB stats API.
If you change an existing endpoint or add a new one, the integration tests in
`api/tests/client.rs` need to be updated.

The tests use `mockito` to serve JSON response fixtures from
`api/tests/responses/`. To add or update a test:

1. Drop a real API response into `api/tests/responses/` as a `.json` file
2. Reference it from a test in `api/tests/client.rs`
3. Run `cargo test -p mlbt-api` to verify

## Pull requests

- Include **screenshots** in the PR description for any visual changes. The TUI
  is screenshot-friendly and it makes review much faster.
- Describe what the change does and why. Link related issues if applicable.
- PRs are squash merged, so individual commits on your branch don't need to be
  perfectly groomed. Focus on a clear PR title and description.
