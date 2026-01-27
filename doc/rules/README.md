# Rust CLI Rules

Best practices distilled from project analysis and refactoring experience.

**Style Philosophy:** When patterns diverge, prefer Ruby/Python idioms—readability over cleverness, flat over nested, explicit over implicit.

## Index

| File | Content |
|------|---------|
| [structure.md](structure.md) | Project layout, CLI architecture, module organization |
| [style.md](style.md) | Naming, Ruby/Python conventions, code patterns |
| [quality.md](quality.md) | Code quality, error handling, avoid duplication |
| [testing.md](testing.md) | Testability, test structure, test types |
| [dependencies.md](dependencies.md) | Crate selection, API clients, observability |
| [architecture.md](architecture.md) | Interface-agnostic design, services, dashboard |
| [output.md](output.md) | UI/output standards with ratatui |
| [checklist.md](checklist.md) | Quick reference checklist and metrics |

## Quick Start

1. Read [structure.md](structure.md) first for project layout
2. Follow [architecture.md](architecture.md) for service design
3. Check [dependencies.md](dependencies.md) before adding crates
4. Review [checklist.md](checklist.md) before committing
