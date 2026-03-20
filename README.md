# Corvo

**A scripting language that replaces Shell.**

Corvo is a modern scripting language designed to be a direct alternative to Bash, Zsh, and Coreutils. It comes with a rich standard library for filesystem operations, HTTP requests, JSON/YAML parsing, cryptography, DNS, and subprocess management -- all built in, no imports needed.

```corvo
# Fetch an API, parse the response, write to disk
var.set("res", http.get(url: "https://api.example.com/data"))
var.set("data", json.parse(map.get(var.get("res"), "response_body")))
fs.write("/tmp/output.json", json.stringify(var.get("data")))
sys.echo("Written ${map.len(var.get("data"))} entries")
```

---

## What makes Corvo different

### No functions. No if/else. No assignment operator.

Corvo deliberately omits three things most languages take for granted. This is not a limitation -- it's the design.

| What's gone | What replaces it |
|---|---|
| `if` / `else` / `elif` | `try { } fallback { }` with `assert_*` |
| `def` / `fn` / `function` | Built-in library calls only |
| `=` assignment | `var.set("name", value)` |

The result is a language with zero ambiguity, no hidden control flow, and scripts that are trivial to read, audit, and maintain.

### Try/Fallback: one construct, two jobs

Corvo uses `try/fallback` for both error handling and conditional logic. Assertions trigger fallback blocks on failure.

```corvo
# Conditional branching via assertions
var.set("port", 8080)

try {
    assert_eq(os.get_env("ENV"), "production")
    var.set("host", "api.prod.example.com")
} fallback {
    var.set("host", "localhost")
}

sys.echo("Connecting to ${var.get("host")}:${var.get("port")}")
```

```corvo
# Chained fallbacks for multiple recovery paths
try {
    var.set("config", fs.read("/etc/app/config.json"))
} fallback {
    var.set("config", fs.read("~/.config/app/config.json"))
} fallback {
    var.set("config", json.stringify({"mode": "default"}))
}
```

```corvo
# Safe operations: error in try block jumps to fallback
try {
    var.set("data", json.parse(fs.read("input.json")))
    var.set("name", map.get(var.get("data"), "user", "anonymous"))
} fallback {
    var.set("name", "anonymous")
}
```

### Compile to a standalone binary

Shell scripts are interpreted. Corvo compiles to a self-contained executable with no runtime dependencies.

```bash
# Write your script
cat > deploy.corvo << 'EOF'
var.set("env", os.get_env("DEPLOY_ENV", "staging"))
sys.echo("Deploying to ${var.get("env")}...")
sys.exec(["rsync", "-avz", "./build/", "server:/app/"], check: true)
sys.echo("Done.")
EOF

# Compile it
corvo --compile deploy.corvo --output deploy

# Run it anywhere, no interpreter needed
./deploy
```

Static variables are pre-executed at compile time and baked into the binary.

### Everything is built in

No `import`. No package manager. No dependency hell.

| Module | Functions |
|---|---|
| `sys` | echo, read_line, sleep, panic, **exec** |
| `fs` | read, write, append, delete, exists, mkdir, list_dir, copy, move, stat |
| `http` | get, post, put, delete |
| `json` | parse, stringify |
| `yaml` | parse, stringify |
| `csv` | parse |
| `xml` | parse |
| `hcl` | parse, stringify |
| `crypto` | hash, hash_file, checksum, encrypt, decrypt, uuid |
| `dns` | resolve, lookup |
| `math` | add, sub, mul, div, mod |
| `os` | get_env, set_env, exec, info |
| `ssh` | exec, scp_upload, scp_download |
| `rsync` | sync |
| `llm` | model, prompt, embed, chat |
| `string` | 12 methods (concat, replace, split, trim, ...) |
| `list` | 11 methods (push, pop, get, join, ...) |
| `map` | 9 methods (keys, values, merge, ...) |
| `number` | 9 methods (abs, floor, ceil, round, sqrt, ...) |

---

## Quick start

### Install

```bash
git clone https://github.com/KeanuReadmes/corvo-lang
cd corvo-lang
cargo build --release
# Binary is at target/release/corvo
```

### Hello world

```bash
corvo -e 'sys.echo("Hello, World!")'
```

### Run a script

```bash
corvo script.corvo
```

### REPL

```bash
corvo --repl
```

### Check syntax

```bash
corvo --check script.corvo
```

---

## Language guide

### Types

Six types, inferred at first assignment:

```corvo
var.set("name", "Alice")         # string
var.set("age", 30)                # number
var.set("active", true)           # boolean
var.set("tags", ["dev", "ops"])   # list
var.set("meta", {"key": "val"})   # map
```

Type is fixed after first `var.set`. Reassigning with a different type is an error.

### Variables

No `=` sign. Two namespaces:

```corvo
# Compile-time constants (baked into binary when compiling)
# static.set() is only allowed inside a prep block, which must come first.
prep {
    static.set("VERSION", "1.0.0")
}

# Runtime variables (mutable)
var.set("counter", 0)
var.set("counter", math.add(var.get("counter"), 1))
sys.echo(var.get("counter"))          # prints 1
sys.echo(static.get("VERSION"))       # prints 1.0.0
```

#### The `prep` block

The `prep` block is used for compile-time setup. It has three strict rules:

1. **Must appear before all other statements.** It is a parse error to place code before a `prep` block.
2. **`static.set()` is only allowed inside `prep`.** Calling `static.set()` anywhere else is a parse error.
3. **Variables created inside `prep` are not available outside it.** Use `static.set()` to pass values to the rest of the program.

```corvo
prep {
    static.set("host", os.get_env("APP_HOST"))
    static.set("port", 8080)
    var.set("tmp", "only visible inside prep")  # not available after prep
}

# static values are accessible everywhere after prep
sys.echo("Connecting to ${static.get("host")}:${static.get("port")}")
```

#### Variable shorthand

`@name` is a shorthand for `var.get("name")`, and `@name = value` is a shorthand for `var.set("name", value)`:

```corvo
# Long form
var.set("host", "localhost")
sys.echo(var.get("host"))

# Equivalent short form using @
@host = "localhost"
sys.echo(@host)

# Shorthand works anywhere var.get() can appear
sys.echo("Connecting to ${@host}:${@port}")
http.get(url: @host)
```

Inside a `browse` block, the key and value bindings are accessed with the `$` prefix instead of `@`, because they are block-scoped and not regular runtime variables:

```corvo
var.set("items", ["a", "b", "c"])

browse(var.get("items"), idx, item) {
    # $idx and $item are the browse-bound bindings — use $ not @
    sys.echo("${$idx}: ${$item}")
}
```

| Shorthand | Equivalent | Scope |
|---|---|---|
| `@name` | `var.get("name")` | runtime variable |
| `@name = val` | `var.set("name", val)` | runtime variable |
| `$name` | browse-bound variable | inside `browse` block only |

### String interpolation

Expressions inside `${...}`:

```corvo
var.set("user", "Alice")
var.set("items", [10, 20, 30])
sys.echo("Hello ${var.get("user")}, you have ${list.len(var.get("items"))} items")
sys.echo("Sum: ${math.add(10, 20)}")
sys.echo("Upper: ${string.to_upper("hello")}")
```

### Loops

Only `loop` (infinite) with `terminate` to exit:

```corvo
var.set("i", 0)
var.set("sum", 0)

loop {
    var.set("sum", math.add(var.get("sum"), var.get("i")))
    var.set("i", math.add(var.get("i"), 1))

    try {
        assert_gt(var.get("i"), 10)
        terminate
    } fallback {
        # continue looping
    }
}

sys.echo("Sum of 1..10 = ${var.get("sum")}")
```

### Browse

`browse` iterates over a list or map, binding a key and value variable for each element. Inside the block, those bindings are accessed with the `$` prefix:

```corvo
# Iterating a list — $idx is the zero-based index, $fruit is the element
var.set("fruits", ["apple", "banana", "cherry"])

browse(var.get("fruits"), idx, fruit) {
    sys.echo("${$idx}: ${$fruit}")
}
# 0: apple
# 1: banana
# 2: cherry
```

```corvo
# Iterating a map — $key is the string key, $val is the associated value
var.set("config", {"host": "localhost", "port": 8080})

browse(var.get("config"), key, val) {
    sys.echo("${$key} = ${$val}")
}
# host = localhost
# port = 8080
```

```corvo
# Using the @ shorthand to pass a regular variable as the iterable
@scores = {"alice": 95, "bob": 87, "carol": 92}

browse(@scores, name, score) {
    sys.echo("${$name}: ${$score}")
}
```

Browse blocks can be nested and support `terminate` to exit early. Pass a browse-bound value to a nested browse using `$name`:

```corvo
@matrix = [[1, 2], [3, 4]]

browse(@matrix, row_idx, row) {
    browse($row, col_idx, cell) {
        sys.echo("[${$row_idx}][${$col_idx}] = ${$cell}")
    }
}
```

| Iterable type | `key` binding | `value` binding |
|---|---|---|
| `list` | numeric index (0, 1, 2, …) | element value |
| `map` | string key | associated value |

### Assertions

Trigger fallback on failure:

| Assertion | Purpose |
|---|---|
| `assert_eq(a, b)` | a == b |
| `assert_neq(a, b)` | a != b |
| `assert_gt(a, b)` | a > b |
| `assert_lt(a, b)` | a < b |
| `assert_match(regex, target)` | target matches regex pattern |

```corvo
try {
    assert_eq(os.get_env("USER"), "root")
    sys.echo("Running as root")
} fallback {
    sys.panic("Must run as root")
}
```

### Data structures

Lists and maps with full method support:

```corvo
# Lists
var.set("nums", [3, 1, 4, 1, 5])
var.set("nums", list.push(var.get("nums"), 9))
sys.echo(list.join(var.get("nums"), ", "))    # "3, 1, 4, 1, 5, 9"
sys.echo(list.len(var.get("nums")))            # 6

# Maps
var.set("config", {"host": "localhost", "port": 8080})
sys.echo(map.get(var.get("config"), "host"))   # "localhost"
var.set("config", map.set(var.get("config"), "debug", true))
sys.echo(json.stringify(var.get("config")))
```

### Subprocess execution

Inspired by Python's `subprocess.run`, `sys.exec` accepts a list of strings where the first element is the program and the remaining elements are its arguments. This avoids shell injection and makes argument passing explicit:

```corvo
# Basic command execution
var.set("result", sys.exec(["ls", "-la", "/tmp"]))
sys.echo(map.get(var.get("result"), "stdout"))

# With input piped to stdin
var.set("upper", sys.exec(["tr", "[:lower:]", "[:upper:]"], input: "hello world"))
sys.echo(map.get(var.get("upper"), "stdout"))  # "HELLO WORLD"

# With working directory and environment
sys.exec(["make", "build"], cwd: "/src/project", env: {"CC": "clang"})

# Timeout protection: timeout causes a runtime error that triggers fallback
try {
    var.set("result", sys.exec(["slow-command"], timeout: 30))
    assert_eq(map.get(var.get("result"), "code"), 0)
    sys.echo("Command succeeded")
} fallback {
    sys.echo("Command timed out or failed")
}

# Strict mode: error on non-zero exit
sys.exec(["deploy.sh"], check: true)

# When shell features like pipelines are needed, pass them via sh -c
var.set("result", sys.exec(["sh", "-c", "df -h / | tail -1"]))
```

| Named arg | Type | Description |
|---|---|---|
| `input` | string | Data piped to stdin |
| `check` | bool | Error if exit code != 0 |
| `timeout` | number | Kill after N seconds |
| `cwd` | string | Working directory |
| `env` | map | Environment variables |

Returns: `{stdout: "...", stderr: "...", code: 0}`

### Named parameters

Python-style keyword arguments for clarity:

```corvo
# Positional
http.get("https://example.com")

# Named (equivalent, clearer)
http.get(url: "https://example.com")

# Mixed
http.post("https://api.example.com", body: '{"key": "val"}', headers: {"Content-Type": "application/json"})
```

---

## Standard library reference

### `sys` -- Core system

| Function | Description |
|---|---|
| `sys.echo(msg...)` | Print to stdout with newline |
| `sys.read_line(prompt?)` | Read a line from stdin |
| `sys.sleep(ms)` | Pause for milliseconds |
| `sys.panic(msg?)` | Raise a runtime error |
| `sys.exec(cmd, ...)` | Execute a process from a list of strings |

### `fs` -- File system

| Function | Description |
|---|---|
| `fs.read(path)` | Read file to string |
| `fs.write(path, content)` | Write/overwrite file |
| `fs.append(path, content)` | Append to file |
| `fs.delete(path)` | Delete file or directory |
| `fs.exists(path)` | Check if path exists |
| `fs.mkdir(path, recursive?)` | Create directory |
| `fs.list_dir(path)` | List directory contents |
| `fs.copy(src, dest)` | Copy file |
| `fs.move(src, dest)` | Move/rename file |
| `fs.stat(path)` | File metadata: `{size, is_dir, permissions, modified_at}` |

### `http` -- HTTP client

| Function | Description |
|---|---|
| `http.get(url, headers?)` | GET request |
| `http.post(url, body, headers?)` | POST request |
| `http.put(url, body, headers?)` | PUT request |
| `http.delete(url, headers?)` | DELETE request |

All return `{status_code, response_body, headers}`.

### `json` -- JSON

| Function | Description |
|---|---|
| `json.parse(str)` | Parse JSON string to Corvo value |
| `json.stringify(value)` | Serialize to pretty-printed JSON |

### `yaml` -- YAML

| Function | Description |
|---|---|
| `yaml.parse(str)` | Parse YAML string |
| `yaml.stringify(value)` | Serialize to YAML |

### `csv` / `xml` / `hcl` -- Data formats

| Function | Description |
|---|---|
| `csv.parse(str, delimiter?)` | Parse CSV with headers |
| `xml.parse(str)` | Parse XML to Corvo value |
| `hcl.parse(str)` | Parse HCL/Terraform config to Corvo value |
| `hcl.stringify(value)` | Serialize to HCL |

### `crypto` -- Cryptography

| Function | Description |
|---|---|
| `crypto.hash(algorithm, data)` | Hash: `md5`, `sha256`, `sha512` |
| `crypto.encrypt(data, key)` | Encrypt (AES-GCM) |
| `crypto.decrypt(data, key)` | Decrypt |
| `crypto.uuid()` | Generate UUID v4 |

### `dns` -- DNS resolution

| Function | Description |
|---|---|
| `dns.resolve(hostname)` | Resolve to list of IPs |
| `dns.lookup(ip)` | Reverse DNS lookup |

### `math` -- Arithmetic

| Function | Description |
|---|---|
| `math.add(a, b)` | a + b |
| `math.sub(a, b)` | a - b |
| `math.mul(a, b)` | a * b |
| `math.div(a, b)` | a / b (errors on zero) |
| `math.mod(a, b)` | a % b (errors on zero) |

### `os` -- Operating system

| Function | Description |
|---|---|
| `os.get_env(key, default?)` | Get environment variable |
| `os.set_env(key, value)` | Set environment variable |
| `os.exec(cmd, args?)` | Simple process execution |
| `os.info()` | Returns `{os, arch, hostname}` |

### `ssh` -- Remote shell

| Function | Description |
|---|---|
| `ssh.exec(host, user, key_path, cmd)` | Execute command on remote host |
| `ssh.scp_upload(host, user, key_path, local_path, remote_path)` | Upload file via SCP |
| `ssh.scp_download(host, user, key_path, remote_path, local_path)` | Download file via SCP |

### `rsync` -- File synchronization

| Function | Description |
|---|---|
| `rsync.sync(source, dest, options?)` | Synchronize files/directories |

### `llm` -- AI language models

| Function | Description |
|---|---|
| `llm.model(name, provider, options?)` | Build a model connection string |
| `llm.prompt(model, prompt, tokens?)` | Execute a prompt against a model |
| `llm.embed(model, text)` | Generate a vector embedding |
| `llm.chat(id, model, messages, tokens?)` | Execute a chat conversation |

### `string` methods

`concat`, `replace`, `split`, `trim`, `contains`, `starts_with`, `ends_with`, `to_lower`, `to_upper`, `len`, `reverse`, `is_empty`

### `list` methods

`push`, `pop`, `get`, `set`, `first`, `last`, `len`, `is_empty`, `contains`, `reverse`, `join`

### `map` methods

`keys`, `values`, `len`, `is_empty`, `has_key`, `get`, `set`, `remove`, `merge`

### `number` methods

`to_string`, `parse`, `is_nan`, `is_infinite`, `is_finite`, `abs`, `floor`, `ceil`, `round`, `sqrt`

---

## CLI reference

```
corvo [OPTIONS] [FILE]
```

| Flag | Short | Description |
|---|---|---|
| `<file>` | | Execute a `.corvo` file |
| `--repl` | `-r` | Interactive REPL |
| `--eval <expr>` | `-e` | Evaluate inline expression |
| `--check <file>` | | Syntax check (no execution) |
| `--compile <file>` | `-c` | Compile to standalone binary |
| `--output <path>` | `-o` | Output path for compiled binary |
| `--debug` | | Use debug build (faster compile) |
| `--version` | `-v` | Print version |

---

## Error system

Corvo has 14 distinct error types, each with a unique exit code:

| Exit code | Error | Trigger |
|---|---|---|
| 1 | Lexing | Invalid tokens |
| 2 | Parsing | Syntax errors |
| 3 | Type | Type mismatch |
| 4 | Runtime | General runtime errors |
| 5 | Assertion | `assert_*` failure |
| 6 | FileSystem | File I/O errors |
| 7 | Network | HTTP/DNS errors |
| 8 | UnknownFunction | Undefined function call |
| 9 | StaticModification | Modifying static at runtime |
| 10 | DivisionByZero | Division by zero |
| 11 | VariableNotFound | Undefined variable |
| 12 | StaticNotFound | Undefined static |
| 13 | Io | General I/O errors |
| 14 | InvalidArgument | Bad function arguments |

All errors include source location (span) information.

---

## More examples

### File processing pipeline

```corvo
var.set("lines", string.split(fs.read("access.log"), "\n"))
var.set("count", 0)

loop {
    try {
        assert_gt(list.len(var.get("lines")), 0)
    } fallback {
        terminate
    }

    var.set("line", list.first(var.get("lines")))
    var.set("lines", list.pop(var.get("lines")))

    try {
        assert_match("ERROR", var.get("line"))
        var.set("count", math.add(var.get("count"), 1))
    } fallback {
        # not an error line, skip
    }
}

sys.echo("Found ${var.get("count")} error lines")
```

### JSON API consumer

```corvo
var.set("res", http.get(url: "https://api.github.com/repos/rust-lang/rust"))

try {
    assert_eq(map.get(var.get("res"), "status_code"), 200)
} fallback {
    sys.panic("API request failed")
}

var.set("repo", json.parse(map.get(var.get("res"), "response_body")))
sys.echo("Name: ${map.get(var.get("repo"), "full_name")}")
sys.echo("Stars: ${map.get(var.get("repo"), "stargazers_count")}")
sys.echo("Language: ${map.get(var.get("repo"), "language")}")
```

### Iterating over a collection with browse

```corvo
# Print every field of a JSON object
var.set("user", json.parse(fs.read("user.json")))

browse(var.get("user"), field, value) {
    sys.echo("${$field}: ${$value}")
}

# Count errors in a log file
var.set("lines", string.split(fs.read("access.log"), "\n"))
var.set("count", 0)

browse(var.get("lines"), idx, line) {
    try {
        assert_match("ERROR", $line)
        var.set("count", math.add(var.get("count"), 1))
    } fallback {
        # not an error line, skip
    }
}

sys.echo("Found ${var.get("count")} error lines")
```

### System health check

```corvo
var.set("info", os.info())
sys.echo("Host: ${map.get(var.get("info"), "hostname")}")
sys.echo("OS: ${map.get(var.get("info"), "os")}/${map.get(var.get("info"), "arch")}")

var.set("disk", sys.exec(["sh", "-c", "df -h / | tail -1"]))
sys.echo("Disk: ${string.trim(map.get(var.get("disk"), "stdout"))}")

var.set("mem", sys.exec(["sh", "-c", "free -h | grep Mem | awk '{print $3 \"/\" $2}'"]))
sys.echo("Memory: ${string.trim(map.get(var.get("mem"), "stdout"))}")

var.set("result", sys.exec(["curl", "-s", "-o", "/dev/null", "-w", "%{http_code}", "https://example.com"], timeout: 5))
try {
    assert_eq(map.get(var.get("result"), "stdout"), "200")
    sys.echo("Health: OK")
} fallback {
    sys.echo("Health: DEGRADED")
}
```

### File sync with error recovery

```corvo
prep {
    static.set("SRC", "/data/important")
    static.set("DEST", "backup-server:/data/important")
}

try {
    sys.exec(
        ["rsync", "-avz", "--delete", "${static.get("SRC")}/", "${static.get("DEST")}/"],
        timeout: 300,
        check: true
    )
    sys.echo("Backup completed successfully")
} fallback {
    sys.echo("Backup failed, sending alert...")
    http.post(
        url: "https://hooks.slack.com/services/xxx",
        body: json.stringify({"text": "Backup failed!"})
    )
}
```

---

## Design philosophy

Corvo's constraints are intentional:

1. **No user-defined functions** forces scripts to be flat, linear, and easy to follow from top to bottom. There is no indirection, no recursion, no call stack to trace.

2. **No if/else** eliminates a class of bugs: dangling else, missing branches, complex conditionals. `try/fallback` with assertions covers every case where you need conditional behavior, and it forces you to handle errors at the same time.

3. **No assignment operator** makes mutation explicit. `var.set("name", value)` is deliberately verbose so that state changes are obvious and searchable.

4. **Batteries included** means no dependency management. A Corvo script is a single file. Always.

5. **Compilable** means your scripts can be distributed as real binaries. No "install this runtime first."

---

## Project structure

```
corvo-lang/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library API
│   ├── error.rs             # Error types (14 variants)
│   ├── span.rs              # Source location tracking
│   ├── lexer/               # Tokenizer
│   ├── parser/              # Recursive descent parser
│   ├── ast/                 # AST node definitions
│   ├── type_system/         # Value type + methods
│   ├── runtime/             # State management
│   ├── compiler/            # Evaluator + standalone builder
│   ├── standard_lib/        # All built-in modules
│   └── repl/                # Interactive REPL
├── examples/                # Example .corvo scripts
├── tests/                   # Integration tests
└── Cargo.toml
```

---

## License

MIT
