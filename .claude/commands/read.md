Smart file reading with outline, interface, around, and diff modes.

## Usage

```bash
hu read src/main.rs                        # Full file
hu read src/main.rs -o                     # Outline (functions, structs, classes)
hu read src/main.rs -i                     # Public interface only
hu read src/main.rs -a 42                  # Lines around line 42
hu read src/main.rs -a 42 -n 20           # 20 context lines around line 42
hu read src/main.rs -d                     # Git diff (vs HEAD)
hu read src/main.rs -d --commit abc123     # Diff against specific commit
```

## Arguments

| Arg | Description |
|-----|-------------|
| `PATH` | File path to read (required) |

## Options

| Flag | Description |
|------|-------------|
| `-o, --outline` | Show file outline (functions, structs, classes) |
| `-i, --interface` | Public interface only (pub items in Rust, exports in JS) |
| `-a, --around` | Show lines around a specific line number |
| `-n, --context` | Context lines for `--around` (default: 10) |
| `-d, --diff` | Show git diff |
| `--commit` | Commit to diff against (default: HEAD) |

## Modes

### Outline (`-o`)
Shows structure: function signatures, struct/class definitions, impl blocks.

### Interface (`-i`)
Shows only public API: `pub fn`, `pub struct`, `pub enum`, exports.

### Around (`-a`)
Shows context around a specific line, useful for investigating errors at known line numbers.

### Diff (`-d`)
Shows git changes, optionally against a specific commit.
