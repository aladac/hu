# Quick Checklist & Metrics

## Metrics Targets

| Metric | Target |
|--------|--------|
| Max file size | <400 lines |
| Max function size | <50 lines |
| Max nesting depth | 3 levels |
| Max parameters | 5 |
| Magic numbers | 0 (use constants) |
| Duplicate code blocks | <3 patterns |

## Pre-Commit Checklist

### Structure
- [ ] Files under 400 lines
- [ ] Functions under 50 lines
- [ ] Nesting depth <= 3 levels

### Style (Ruby/Python)
- [ ] Predicates use `is_`, `has_`, `can_` prefixes
- [ ] Iterator chains over imperative loops
- [ ] Early returns to flatten logic
- [ ] Builder pattern for complex construction
- [ ] All types derive `Debug`

### Quality
- [ ] No magic numbers (use constants)
- [ ] No hardcoded URLs
- [ ] All errors propagated with context
- [ ] Common patterns extracted to helpers

### Testing
- [ ] Code is testable (logic separated from I/O)
- [ ] External deps use traits (mockable)
- [ ] Unit tests inline with `#[cfg(test)]` modules
- [ ] Integration tests in `tests/` directory
- [ ] No "hard to test" code accepted

### Tooling
- [ ] `clippy.toml` and `rustfmt.toml` configured
- [ ] CI runs fmt, clippy, and tests

## Commands

```bash
just check      # fmt + clippy
just test       # run all tests
just build      # build debug
just release    # build release
```
