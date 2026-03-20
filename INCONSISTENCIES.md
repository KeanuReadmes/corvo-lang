# Corvo Language Implementation Inconsistencies

This document lists all inconsistencies found between the official language specification (IMPLEMENTATION.md) and the actual documentation/examples in the repository.

## Critical Violations

### 1. **Illegal `let` Assignment Syntax** (README.md, Lines 9-10)

**Location:** README.md, opening example code block  
**Issue:** Uses `let` for variable assignment, which violates the core design principle.

```corvo
# ❌ INCORRECT (from README.md):
let res = http.get(url: "https://api.example.com/data")
let data = json.parse(res.response_body)

# ✅ CORRECT (per IMPLEMENTATION.md):
var.set("res", http.get(url: "https://api.example.com/data"))
var.set("data", json.parse(map.get(var.get("res"), "response_body")))
```

**Specification:** IMPLEMENTATION.md explicitly states:
> **NO ASSIGNMENT OPERATORS:** Do not use `=` for assignment. State is strictly managed via `var.set()` and `static.set()`.

**Impact:** This is in the very first code example, setting the wrong expectation for new users.

---

### 2. **Direct Property Access Instead of map.get()** (README.md, Multiple Locations)

**Locations:**
- Line 10: `res.response_body`
- Line 265: `var.get("result").stdout`
- Line 269: `upper.stdout`
- Line 507: `var.get("res").status_code`
- Line 512-515: Multiple `.response_body` accesses

**Issue:** Uses dot notation for map property access instead of `map.get()`.

```corvo
# ❌ INCORRECT:
var.get("res").status_code
var.get("result").stdout

# ✅ CORRECT:
map.get(var.get("res"), "status_code")
map.get(var.get("result"), "stdout")
```

**Specification:** IMPLEMENTATION.md defines `map.get(target: map, key: string, default: any) -> any` as the only way to access map values.

---

### 3. **Try/Fallback Without Assertions** (README.md, Multiple Examples)

**Locations:**
- Lines 51-57: Chained fallbacks for file reading
- Lines 62-67: Safe operations example
- Lines 275-279: Timeout protection
- Lines 546-559: File sync with error recovery

**Issue:** Multiple `try/fallback` blocks have no `assert_*` statements in the try block, relying purely on runtime errors.

```corvo
# ❌ QUESTIONABLE PATTERN (from README.md lines 51-57):
try {
    var.set("config", fs.read("/etc/app/config.json"))
fallback {
    var.set("config", fs.read("~/.config/app/config.json"))
fallback {
    var.set("config", json.stringify({"mode": "default"}))
}
```

**Specification:** IMPLEMENTATION.md states:
> Control flow is strictly handled via `try { ... } fallback { ... }` **combined with `assert_*` commands**.
>
> Execution proceeds linearly until an `assert_*` fails or a command errors out

**Analysis:** While the spec does mention "a command errors out," the emphasis throughout is on explicit assertions for control flow. The README examples rely heavily on implicit error triggering, which contradicts the stated design philosophy of "zero ambiguity, no hidden control flow."

---

### 4. **Missing assert_* in Timeout Example** (README.md, Lines 275-279)

**Location:** README.md, subprocess execution section

```corvo
# ❌ INCOMPLETE:
try {
    sys.exec("slow-command", timeout: 30)
fallback {
    sys.echo("Command timed out or failed")
}
```

**Issue:** No assertion to check if the command succeeded or distinguish timeout from other failures.

**Better pattern:**
```corvo
try {
    var.set("result", sys.exec("slow-command", timeout: 30))
    assert_eq(map.get(var.get("result"), "code"), 0)
    sys.echo("Command succeeded")
fallback {
    sys.echo("Command timed out or failed")
}
```

---

## Minor Inconsistencies

### 5. **Missing Type Methods in README.md**

**Issue:** The README.md lists method names but not complete signatures.

- **README.md (Line 412):** `string` has "12 methods"
- **IMPLEMENTATION.md:** Lists 8 string methods explicitly
- **README.md (Line 416):** `list` has "11 methods"  
- **IMPLEMENTATION.md:** Lists 6 list methods explicitly

**Impact:** Users cannot know exact function signatures without consulting IMPLEMENTATION.md.

---

### 6. **Conflicting `llm` Module Status**

**IMPLEMENTATION.md:** Includes extensive `llm` module documentation:
- `llm.model(name, provider, options)`
- `llm.prompt(model, prompt, tokens)`
- `llm.embed(model, text)`
- `llm.chat(id, model, messages, tokens)`

**README.md:** No mention of `llm` module in the standard library table (Line 96-112).

**Issue:** Unclear if this is implemented, planned, or deprecated.

---

### 7. **SSH/Rsync Module Missing from README**

**IMPLEMENTATION.md:** Documents `ssh` and `rsync` modules:
- `ssh.exec()`
- `ssh.scp_upload()`
- `ssh.scp_download()`
- `rsync.sync()`

**README.md:** No mention of these modules.

**Impact:** Useful functionality is undocumented in the main README.

---

### 8. **HCL Parsing Module Missing from README**

**IMPLEMENTATION.md:** 
```
hcl.parse(data: string) -> map  # Essential for parsing Terraform configs
hcl.stringify(data: map) -> string
```

**README.md:** Only mentions `csv` and `xml` under "Data formats."

---

### 9. **Inconsistent Crypto Documentation**

**IMPLEMENTATION.md:**
- `crypto.encrypt()` uses AES-GCM

**README.md (Line 380):**
- `crypto.encrypt()` uses "XOR + base64"

**Issue:** These are completely different encryption methods with vastly different security properties.

---

### 10. **Repository URL Mismatch**

**README.md (Line 121):**
```bash
git clone https://github.com/anomalyco/corvo-lang
```

**Actual repository:**
```
KeanuReadmes/corvo-lang
```

**Issue:** Installation instructions point to wrong repository.

---

## Documentation Clarity Issues

### 11. **Ambiguous assert_match Syntax**

**README.md (Line 229):**
```
assert_match(regex, str)  # str matches regex pattern
```

**IMPLEMENTATION.md:**
```
assert_match(regex: string, target: string)
```

**Issue:** Parameter order and naming differ between documents.

---

### 12. **Subprocess Execution API Confusion**

The README shows both `sys.exec()` and `os.exec()`:
- **Line 79:** `sys.exec("rsync -avz ./build/ server:/app/", check: true)`
- **Line 407:** `os.exec(cmd)` described as "Simple command execution"

**IMPLEMENTATION.md** lists:
- `sys.exec(cmd, ...)` with no parameters defined
- `os.exec(cmd: string, args: list) -> map`

**Issue:** Unclear which function supports which parameters (timeout, check, cwd, env, shell).

---

## Summary

**Total Issues Found:** 12

**Critical (Breaks Core Design):** 4
- `let` assignment syntax
- Direct property access instead of `map.get()`
- Try/fallback without assertions
- Missing assertions in control flow

**Medium (Functionality Mismatch):** 5
- Missing modules in README
- Conflicting crypto implementation details
- Wrong repository URL
- API confusion between sys.exec/os.exec

**Minor (Documentation):** 3
- Incomplete method signatures
- Ambiguous parameter naming
- Type method count mismatches

---

## Recommendations

1. **Remove ALL `let` syntax from README.md immediately** - This fundamentally contradicts the language design.

2. **Replace dot notation with `map.get()` throughout examples** - Maintain consistency with the no-operator philosophy.

3. **Add explicit assertions to try/fallback blocks** - Even if runtime errors work, the examples should demonstrate the intended pattern.

4. **Synchronize module documentation** - Decide which modules are actually implemented and document them consistently.

5. **Fix crypto.encrypt() documentation** - Clarify whether it's XOR or AES-GCM (security implications are significant).

6. **Consolidate subprocess execution API** - Pick one function (sys.exec or os.exec) for advanced features, document clearly.

7. **Update repository URL** - Point to the actual repository in installation instructions.