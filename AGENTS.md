# Corvo — Contributor & Agent Guide

This file is the authoritative guide for human contributors and AI coding agents
working on the Corvo language implementation. Read it before making any changes.

---

## Repository layout

```
src/
  ast/               Abstract Syntax Tree node definitions
  compiler/
    builder.rs       Code-generator: emits Rust source and drives cargo build
    evaluator.rs     Compile-time evaluator for `prep { }` blocks
  lexer/             Tokeniser (tokenizer.rs) and token definitions (token.rs)
  parser/            Recursive-descent parser
  runtime/           Runtime state (variables, statics)
  standard_lib/      Built-in modules: sys, fs, http, net, json, yaml, xml, csv,
                     hcl, math, os, args, crypto, dns, llm
  type_system/       Value enum, type methods (list.*, map.*, string.*, …)
  diagnostic.rs      Linter + pretty error rendering
  lib.rs             Public API (run_source, run_source_with_state, …)
  main.rs            CLI entry point
examples/            .corvo example scripts, one per feature area
coreutils/           Larger stdlib-driven demos (e.g. GNU-oriented `ls.corvo`)
tests/
  integration_test.rs  Integration test suite (≥ 60 tests)
```

---

## Development workflow

### 1. Build

```bash
cargo build           # debug build (fast)
cargo build --release # release build (used for --compile)
```

### 2. Run the linter on all examples

**Always run this after any change that touches `src/diagnostic.rs` or any
`src/type_system/` or `src/standard_lib/` file.**

```bash
# Build a fresh release binary first, then lint every example.
cargo build --release
for f in examples/*.corvo coreutils/*.corvo; do
    result=$(target/release/corvo --lint "$f" 2>&1)
    if echo "$result" | grep -q "^error:"; then
        echo "LINT FAIL: $f"
        echo "$result"
    fi
done
```

All files in `examples/` and `coreutils/` must report `no issues found`. If you add or rename a built-in
function, update `KNOWN_FUNCTIONS` in `src/diagnostic.rs` to match the
implementations in `src/type_system/type_methods.rs` and
`src/standard_lib/*.rs`.

### 3. Clippy

Run clippy the same way CI does — denying all warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Fix every warning before committing. Never suppress a warning with `#[allow]`
unless it is genuinely unavoidable and the suppression is accompanied by a
comment explaining why.

### 4. Formatting

Always run `cargo fmt` before committing. CI enforces this with `cargo fmt --check`.

```bash
cargo fmt
```

### 5. Tests

```bash
cargo test --all-features          # unit + integration tests
cargo test --lib                   # unit tests only
cargo test --test integration_test # integration tests only
```

All tests must pass before opening a PR.

### 6. Full pre-commit checklist

Run these commands in order before every commit:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
for f in examples/*.corvo coreutils/*.corvo; do target/release/corvo --lint "$f"; done
cargo test --all-features
# Optional but recommended before pushing ls changes (requires Docker):
# ./scripts/ls-parity-docker.sh --require-docker
# Wider flag comparison (reports PASS/FAIL per option group; may include expected gaps):
# ./scripts/ls-parity-matrix.sh
cargo fmt
```

---

## Key invariants to preserve

### `KNOWN_FUNCTIONS` must match the runtime

`src/diagnostic.rs` contains a `KNOWN_FUNCTIONS` static slice used by
`corvo --lint`. It **must** stay in sync with:

- `src/type_system/type_methods.rs` — `list.*`, `map.*`, `string.*`,
  `number.*` methods
- `src/standard_lib/*.rs` — every `sys.*`, `fs.*`, `http.*`, `net.*`, `os.*`,
  `math.*`, `time.*`, `crypto.*`, `dns.*`, `json.*`, `yaml.*`, `xml.*`, `csv.*`,
  `hcl.*`, `llm.*` function

When you add, rename, or remove a built-in, update `KNOWN_FUNCTIONS`
**in the same commit**.

### Static variables are obfuscated

`prep { static.set("KEY", value) }` values are serialised to JSON,
XOR-encrypted with a random per-compilation key, and embedded as raw byte
arrays in the generated binary. They must **never** appear as plain-text
string literals in `main.rs`. See `src/compiler/builder.rs` →
`generate_main_rs` and `src/lib.rs` → `load_statics_from_encrypted_bytes`.

### Example files are the integration surface

Every file under `examples/` and `coreutils/` must pass `corvo --lint`. They
demonstrate the public API and are also useful for manual regression testing.
Keep them in sync with the runtime.

---

## Adding a new built-in function

1. Implement it in the appropriate `src/standard_lib/<module>.rs` or
   `src/type_system/type_methods.rs` file.
2. Add its fully-qualified name (e.g. `"list.count"`) to `KNOWN_FUNCTIONS` in
   `src/diagnostic.rs`.
3. Add or update an example under `examples/` (or `coreutils/` when appropriate).
4. Add unit tests next to the implementation and integration tests in
   `tests/integration_test.rs`.
5. Run the full pre-commit checklist above.
6. Document the new function in `CHEATSHEET.md` and `IMPLEMENTATION.md`.

---

## CI

The `.github/workflows/ci.yml` pipeline runs on every push and PR to `main`:

| Job | Command |
|---|---|
| Format | `cargo fmt --check` |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` |
| Test | `cargo test --all-features` (Linux, macOS, Windows) |
| GNU ls parity | `./scripts/ls-parity-docker.sh --require-docker` (Linux + Docker only) |
| GNU ls matrix | `./scripts/ls-parity-matrix.sh` (optional; many flag combos vs GNU `ls`) |
| Build | `cargo build --release --all-features` (Linux, macOS, Windows) |

All jobs must be green before merging.
