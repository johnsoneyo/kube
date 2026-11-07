# kube

A lightweight Kubernetes companion CLI written in Rust. It builds on `kubectl` without introducing a shelling-out layer and keeps the command parsing, context selection, and execution concerns separated.

## Features

- `pods list [filter]` to list pod resources with optional filtering
- `logs follow <pod> [--tail N]` to stream a pod log stream
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
# list pods in the current context
cargo run -- pods list

# list pods with a namespace and context override
cargo run -- --context prod --namespace payments pods list api

# stream pod logs
cargo run -- logs follow api --tail 100

# display the current context
cargo run -- ctx current

# switch to a different kube context
cargo run -- ctx use staging

# view overall status
cargo run -- status overview
```

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

- `follow` uses a streamed process and prints each log line as it arrives.
- `show` reads the log stream once and exits after completion.

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

This summarizes pod health and deployment availability from `kubectl get ...` output without requiring additional dependencies.

## Architecture

- `src/main.rs` launches the CLI and dispatches commands.
- `src/cli/` contains the Clap command definitions and handlers.
- `src/context.rs` stores the shared namespace and context override.
- `src/di.rs` defines the `Executor` abstraction used for testable `kubectl` interaction.

## Development

```bash
cargo fmt
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

## CI

GitHub Actions runs format, compile, lint, and test checks on each push and pull request via `.github/workflows/ci.yml`.

## License

This project is licensed under the MIT License. See the repository license file for the full text.

