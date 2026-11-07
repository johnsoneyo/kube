# Changelog

## 0.1.1 - 2026-11-07

### Added
- Namespace-aware status scope handling so status commands behave consistently with and without `--namespace`.
- JSON-based Kubernetes status parsing for pod and deployment health summaries using `kubectl get ... -o json`.
- Better CLI and executor test coverage for context propagation, malformed JSON, empty resource lists, and kubectl failures.
- Documentation updates describing scope semantics, status behavior, and development verification commands.

### Fixed
- Resolved status command conflicts between `-n` and `-A` by making all-namespaces the explicit default when no namespace is provided.
- Replaced brittle human-readable table parsing with structured Kubernetes data classification.
- Improved kubectl error propagation so failed commands surface meaningful stderr context.
- Hardened streaming execution to detect process failures without panicking.

### Verified
- `cargo fmt --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

All of the above checks are passing in the current repository state.
