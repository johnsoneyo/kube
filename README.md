# kube

A lightweight Kubernetes companion CLI written in Rust. It wraps `kubectl` behind a small executor abstraction so the command layer can be tested without a live cluster.

## Features

- `pods list [filter]` to list pod resources with optional filtering
- `logs follow <pod> [--tail N]` to stream pod logs
- `logs show <pod> [--tail N]` to fetch logs once
- `ctx current` to show the active Kubernetes context
- `ctx list` to list configured contexts
- `ctx use <name>` to switch the active context
- `status overview|pods|deployments|issues` to summarize cluster health

## Prerequisites

- Rust stable toolchain
- Kubernetes cluster access or a valid `kubectl` config
- `kubectl` installed and on `PATH`

## Installation

```bash
cargo build --release
```

## Usage

```bash
# list pods in the active context
cargo run -- pods list

# list pods in a specific namespace and context
cargo run -- --context prod --namespace payments pods list api

# stream pod logs
cargo run -- logs follow api --tail 100

# display the current context
cargo run -- ctx current

# switch to a different kubectl context
cargo run -- ctx use staging

# view overall status in all namespaces by default
cargo run -- status overview

# scope status checks to a namespace
cargo run -- --namespace payments status overview
```

## Scope semantics

`kubectl` global flags are applied consistently:

- if `--context` is set, it is forwarded to every kubectl call
- if `--namespace` is set, the status commands operate only within that namespace
- if no namespace is provided, the status commands default to all namespaces with `-A`

This prevents conflicts between `-n` and `-A` while keeping cluster-wide status checks intuitive.

## Command reference

### Pods

```bash
kube pods list [filter]
```

- `filter` is optional and matches against the pod name or row content.
- Global flags `--context` and `--namespace` are forwarded to `kubectl` automatically.

### Logs

```bash
kube logs follow <pod-name> [--tail N]
kube logs show <pod-name> [--tail N]
```

- `follow` streams stdout from the kubectl process and prints each log line as it arrives
- `show` fetches the log stream once and exits after completion
- nonzero kubectl exit codes are surfaced as errors with stderr context when available

### Context

```bash
kube ctx current
kube ctx list
kube ctx use <name>
```

### Status

```bash
kube status overview
kube status pods
kube status deployments
kube status issues
```

The status commands use `kubectl get ... -o json` and parse the structured payload instead of brittle table text. This makes classification resilient to formatting changes and easier to unit test with JSON fixtures.

## Architecture

- `src/main.rs` launches the CLI and dispatches commands
- `src/cli/` contains the Clap command definitions and handlers
- `src/context.rs` stores the shared namespace and context override
- `src/di.rs` defines the `Executor` abstraction used for testable `kubectl` interaction
- `src/cli/status.rs` contains the JSON-based status parsing and health classification functions

## Development

```bash
cargo fmt
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

## CI

GitHub Actions runs format, compile, lint, and test checks on each push and pull request via `.github/workflows/ci.yml`.

## License

This project is licensed under the MIT License. See the repository license file for the full text.

