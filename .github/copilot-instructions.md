# GitHub Copilot Instructions — Corvo Language

This file gives GitHub Copilot (and other AI coding assistants) the context
needed to contribute correctly to the Corvo language implementation.

See **`AGENTS.md`** for the full contributor workflow, including the mandatory
pre-commit checklist.

---

## What is Corvo?

Corvo is a modern scripting language that compiles to standalone Rust binaries.
It deliberately omits three constructs found in most languages:

- **No user-defined functions** — only built-in library calls
- **No if/else** — branching uses `try { } fallback { }` with `assert_*`
- **No assignment operator** — state is managed via `var.set()` / `static.set()`

The implementation is pure Rust (Rust 2021 edition). Corvo scripts are lexed,
parsed, and either interpreted directly or compiled to a temporary Rust crate
that is built by `cargo`.

---

## Mandatory quality gates

Run these before every commit — the same checks that CI enforces:

```bash
# 1. Clippy (zero warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# 2. Build release binary
cargo build --release

# 3. Lint every example (all must pass with no errors)
for f in examples/*.corvo; do
    result=$(target/release/corvo --lint "$f" 2>&1)
    if echo "$result" | grep -q "^error:"; then
        echo "LINT FAIL: $f"
        echo "$result"
    fi
done

# 4. Tests
cargo test --all-features

# 5. Formatting (must be last — formats in place)
cargo fmt
```

---

## Key files to know

| File | Purpose |
|---|---|
| `src/diagnostic.rs` | Linter; contains `KNOWN_FUNCTIONS` — must stay in sync with the runtime |
| `src/type_system/type_methods.rs` | Implementations of `list.*`, `map.*`, `string.*`, `number.*` |
| `src/standard_lib/` | All built-in modules (`sys`, `fs`, `http`, `os`, `math`, …) |
| `src/compiler/builder.rs` | Code-generator; produces the `main.rs` compiled into the binary |
| `src/lib.rs` | Public API; includes `load_statics_from_encrypted_bytes` |
| `examples/*.corvo` | One example per feature — all must pass `corvo --lint` |
| `tests/integration_test.rs` | Integration test suite |

---

## Critical invariants

### Keep `KNOWN_FUNCTIONS` in sync

Every time you add, rename, or remove a built-in function, update the
`KNOWN_FUNCTIONS` slice in `src/diagnostic.rs` in the same commit. Failure to
do so causes `corvo --lint` to report false positives on example files.

The list must cover every function implemented in:
- `src/type_system/type_methods.rs`
- `src/standard_lib/*.rs`

### Never embed static values as plaintext

`static.set()` values baked via `prep { }` blocks are serialised to JSON,
XOR-encrypted, and stored as raw byte arrays in the compiled binary. They must
not appear as readable strings. See `generate_main_rs` in
`src/compiler/builder.rs` and `load_statics_from_encrypted_bytes` in
`src/lib.rs`.

---

## Writing Corvo code (examples and tests)

- Use `var.set("name", value)` / `var.get("name")` for runtime variables.
- Use `static.set("name", value)` **only inside a `prep { }` block**.
- Use `try { ... } fallback { ... }` with `assert_*` for branching.
- Use `${expr}` for string interpolation.
- Type methods return new values; they do not mutate in place.
- `list.new()` and `map.new()` create empty collections.
- `@name` is shorthand for `var.get("name")`.

---

## Adding a built-in function

1. Implement in `src/type_system/type_methods.rs` or the relevant
   `src/standard_lib/<module>.rs`.
2. Add the fully-qualified name to `KNOWN_FUNCTIONS` in `src/diagnostic.rs`.
3. Add or update the matching example in `examples/`.
4. Add tests in `tests/integration_test.rs`.
5. Document in `CHEATSHEET.md`.
6. Run the full quality-gate checklist above.
