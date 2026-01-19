# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## !!! CRITICAL - AWS SAFETY RULES !!!

**NEVER run write/execute/modify commands - this tool is READ-ONLY!**
**NEVER start pipelines, create resources, terminate instances, or modify anything!**
**ONLY use `-e dev` for any EKS testing!**
**ALL AWS access is for STATUS REPORTING ONLY:**
- Pipelines: list, get status, view logs - NO start/stop/approve
- EC2: list instances, describe - NO create/start/stop/terminate  
- Secrets: list names only - NO read values, NO write
- S3: list buckets - NO read/write objects
- EKS: list pods, view logs - NO delete/modify

## Project Overview

**hu** (husky) is a dev workflow CLI that unifies: EKS pod access, Jira tickets, GitHub PRs/Actions, and AWS pipelines.

## Commands

All commands use `just` (see justfile):

```bash
just check      # Run all checks (fmt, clippy)
just fmt        # Format with cargo fmt
just clippy     # Lint with clippy
just build      # Build debug
just release    # Build release
just install    # Install locally via cargo
just bump       # Bump version (see scripts/bump.sh)
```

Run the tool directly:
```bash
cargo run                           # List pods (auto-detects env from kubectl context)
cargo run -- --pod 1                # Connect to pod #1
cargo run -- -e prod -t api         # List api pods on prod
cargo run -- --log                  # Tail logs from all matching pods
```

Or after `just install`:
```bash
hu                                  # List pods
hu --pod 1                          # Connect to pod #1
hu -e prod -t api                   # List api pods on prod
hu --log                            # Tail logs from all matching pods
```

## Dependencies

Rust 1.70+ with these crates:
- **clap**: CLI argument parsing with derive macros
- **colored**: Terminal colors
- **comfy-table**: Pretty table display
- **indicatif**: Progress spinners
- **anyhow/thiserror**: Error handling
- **ctrlc**: Signal handling for log tailing

## Architecture

Single-file CLI (`src/main.rs`) built with clap. Key components:

- **Environment enum**: Defines prod/dev/stg with associated cluster names and emojis
- **AWS integration**: Checks SSO session, triggers login if needed, updates kubeconfig
- **Pod operations**: Lists pods by namespace/pattern, displays in comfy-table, execs into selected pod with custom PS1 prompt
- **Log tailing**: Parallel log streaming from multiple pods using std::thread with color-coded output by pod

## Settings

All settings are hardcoded in `src/main.rs`:

- **Environment::cluster()**: Maps environment to EKS cluster names (`prod-eks`, `eks-dev`, `eks-stg`)
- **Default namespace**: `cms` (clap default)
- **Default pod type**: `web` (clap default)
- **Default region**: `us-east-1` (in update_kubeconfig)
- **Default log path**: `/app/log/{environment}.log`
- **ANSI_COLORS**: Colors for multi-pod log output

## Test Structure

Tests are in `tests/` directory (not inline):
- `tests/unit.rs` → `tests/unit_tests/`
- `tests/integration.rs` → `tests/integration_tests/`

## Claude Code Settings

Project-local settings in `.claude/settings.json` auto-allow all Bash commands.
