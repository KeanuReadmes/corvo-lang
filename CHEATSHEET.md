# Corvo Standard Library Cheatsheet

Quick reference for every built-in function in the Corvo standard library.  
Functions are grouped by module. Parameter names in `[brackets]` are optional.

---

## `sys` — System I/O

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `sys.echo` | `message: string` | `null` | Print a value to stdout |
| `sys.read_line` | `[prompt: string]` | `string` | Read a line from stdin |
| `sys.sleep` | `ms: number` | `null` | Pause execution for `ms` milliseconds |
| `sys.panic` | `[message: string]` | *(exits)* | Terminate with a non-zero exit code |
| `sys.exit` | `[code: number]` | *(exits)* | Terminate with exit code `code` (default `0`) |
| `sys.exec` | `cmd: list[string]` | `map{stdout, stderr, code}` | Run an external command (no shell) |

**Example file:** [`examples/sys_example.corvo`](examples/sys_example.corvo)

---

## `os` — Operating System

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `os.get_env` | `key: string, [default: string]` | `string` | Read an environment variable |
| `os.set_env` | `key: string, value: string` | `null` | Set an environment variable |
| `os.exec` | `cmd: string` | `map{stdout, stderr, code}` | Run a shell command via `sh -c` |
| `os.info` | *(none)* | `map{os, arch, hostname}` | Return OS / architecture / hostname |
| `os.argv` | *(none)* | `list[string]` | Script arguments (after `corvo script.corvo …` or after the compiled binary name) |

**Example file:** [`examples/os_example.corvo`](examples/os_example.corvo)

---

## `args` — Command-line parsing

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `args.parse` | `argv: list[string], config?: map` | `map{positional, options, plus?, at_servers?}` | Generic configurable argv parser. Supports GNU coreutils style (short clusters, long options, `--key=val`, aliases, accumulate), dnsutils/dig style (`+flag`, `+noflag`, `+key=val`, `@server`), and usbutils style (colon-compound values). All config keys are optional. |
| `args.scan` | `argv: list[string]` | `map{positional, options}` | Zero-config wrapper around `args.parse`. Backward-compatible with existing scripts. |

### `args.parse` config map keys

| Key | Type | Description |
|---|---|---|
| `"aliases"` | `map` | raw-key → semantic output key (e.g. `"l": "long"`) |
| `"short_values"` | `list[string]` | short chars that consume a glued tail or next token as a value |
| `"long_values"` | `list[string]` | long flag names (normalised) that require a value via `=` or next token |
| `"long_optional_values"` | `list[string]` | long flags whose value is only accepted via `=` (defaults to `"always"`) |
| `"accumulate"` | `list[string]` | output keys where repeated values build a list instead of overwriting |
| `"plus_flags"` | `bool` | enable dig-style `+flag` / `+noflag` / `+key=val` collected into `"plus"` map |
| `"at_tokens"` | `bool` | collect `@server` tokens into `"at_servers"` list |
| `"permute"` | `bool` (default `true`) | GNU mode: interleave options and operands; `false` = stop at first positional (POSIX) |

**Example files:** [`examples/args.corvo`](examples/args.corvo), [`examples/args_parse.corvo`](examples/args_parse.corvo), [`examples/coreutils_ls.corvo`](examples/coreutils_ls.corvo)

---

## `math` — Arithmetic

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `math.add` | `a: number, b: number` | `number` | `a + b` |
| `math.sub` | `a: number, b: number` | `number` | `a - b` |
| `math.mul` | `a: number, b: number` | `number` | `a * b` |
| `math.div` | `a: number, b: number` | `number` | `a / b` — error if `b == 0` |
| `math.mod` | `a: number, b: number` | `number` | `a % b` — error if `b == 0` |
| `math.max` | `a: number, b: number, …` | `number` | Largest argument (two or more numbers) |
| `math.human_bytes` | `bytes: number, [si: bool]` | `string` | Human-readable size (`1024`‑based by default; `si: true` uses `1000`‑based prefixes) |

**Example file:** [`examples/math_example.corvo`](examples/math_example.corvo)

---

## `time` — Timestamps

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `time.format_local` | `seconds: number, [nanoseconds: number], format: string` | `string` | Format Unix time with `chrono` strftime in the **local** timezone (honours `TZ`) |
| `time.unix_now` | *(none)* | `number` | Current time as seconds since the Unix epoch |

---

## `fs` — File System

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `fs.read` | `path: string` | `string` | Read entire file as a string |
| `fs.write` | `path: string, content: string` | `bool` | Write (overwrite) a file |
| `fs.append` | `path: string, content: string` | `bool` | Append to a file |
| `fs.delete` | `path: string` | `bool` | Delete a file or directory |
| `fs.exists` | `path: string` | `bool` | Check whether a path exists |
| `fs.mkdir` | `path: string, [recursive: bool]` | `bool` | Create a directory |
| `fs.list_dir` | `path: string` | `list[string]` | List directory entries |
| `fs.copy` | `src: string, dest: string` | `bool` | Copy a file |
| `fs.move` | `src: string, dest: string` | `bool` | Move / rename a file |
| `fs.stat` | `path: string` | `map{size, is_dir, permissions, modified_at}` | Get file metadata |
| `fs.read_meta` | `path: string, [follow_symlinks: bool]` | `map` | Rich metadata (mode, inode, nlink, uid/gid, user/group, symlink target, blocks, times, …) — Unix fields are zeros / placeholders on non-Unix |
| `fs.read_dir_meta` | `path: string, [follow_symlinks: bool]` | `list[map]` | Directory entries with the same metadata shape as `fs.read_meta` |
| `fs.read_link` | `path: string` | `string` | Target path of a symbolic link |

**Example file:** [`examples/fs_example.corvo`](examples/fs_example.corvo)

---

## `http` — HTTP Client

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `http.get` | `url: string, [headers: map]` | `map{status_code, response_body, headers}` | HTTP GET request |
| `http.post` | `url: string, [body: string]` | `map{status_code, response_body}` | HTTP POST request |
| `http.put` | `url: string, [body: string]` | `map{status_code, response_body}` | HTTP PUT request |
| `http.delete` | `url: string` | `map{status_code, response_body}` | HTTP DELETE request |

**Example file:** [`examples/http_example.corvo`](examples/http_example.corvo) *(requires network)*

---

## `net` — TCP sockets

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `net.tcp_listen` | `address: string` | `map` | `tcp_listener` handle: `kind`, `id`, `local_addr` |
| `net.tcp_accept` | `listener: map` | `map` | Blocks; `tcp_stream` handle: `kind`, `id`, `local_addr`, `peer_addr` |
| `net.tcp_close_listener` | `listener: map` | `null` | Close listener; handle is invalid after |
| `net.tcp_connect` | `address: string` | `map` | Client `tcp_stream` handle |
| `net.tcp_read` | `stream: map, max_bytes: number` | `string` | Up to `max_bytes`; non-UTF-8 is lossy-decoded |
| `net.tcp_write` | `stream: map, data: string` | `null` | Send bytes of `data` |
| `net.tcp_close` | `stream: map` | `null` | Close stream; handle is invalid after |

**Example file:** [`examples/net_tcp.corvo`](examples/net_tcp.corvo)

---

## `dns` — DNS Lookup

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `dns.resolve` | `hostname: string` | `list[string]` | Resolve hostname to IP addresses |
| `dns.lookup` | `ip: string` | `string` | Reverse-DNS: IP address to hostname |

**Example file:** [`examples/dns_example.corvo`](examples/dns_example.corvo) *(requires network)*

---

## `crypto` — Cryptography

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `crypto.hash` | `algorithm: string, data: string` | `string` | Hash a string (`md5`, `sha256`, `sha512`) |
| `crypto.hash_file` | `algorithm: string, path: string` | `string` | Hash the contents of a file |
| `crypto.checksum` | `path: string` | `string` | SHA-256 checksum of a file |
| `crypto.encrypt` | `secret: string, value: string` | `string` | XOR-encrypt and base64-encode |
| `crypto.decrypt` | `secret: string, value: string` | `string` | Base64-decode and XOR-decrypt |
| `crypto.uuid` | *(none)* | `string` | Generate a UUID v4 |

**Example file:** [`examples/crypto_example.corvo`](examples/crypto_example.corvo)

---

## `json` — JSON

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `json.parse` | `text: string` | `any` | Parse a JSON string into a value |
| `json.stringify` | `value: any` | `string` | Serialize a value to a JSON string |

**Example file:** [`examples/json_example.corvo`](examples/json_example.corvo)

---

## `yaml` — YAML

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `yaml.parse` | `text: string` | `any` | Parse a YAML string into a value |
| `yaml.stringify` | `value: any` | `string` | Serialize a value to a YAML string |

**Example file:** [`examples/yaml_example.corvo`](examples/yaml_example.corvo)

---

## `csv` — CSV

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `csv.parse` | `text: string, [delimiter: string]` | `list[list[string]]` | Parse CSV data (first row is treated as headers and consumed) |

**Example file:** [`examples/csv_example.corvo`](examples/csv_example.corvo)

---

## `xml` — XML

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `xml.parse` | `text: string` | `map` | Parse an XML string into a map |

**Example file:** [`examples/xml_example.corvo`](examples/xml_example.corvo)

---

## `env` — .env File Parser

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `env.parse` | `text: string` | `map` | Parse `.env`-format text into a key/value map |

**Example file:** [`examples/env_example.corvo`](examples/env_example.corvo)

---

## `hcl` — HCL (HashiCorp Configuration Language)

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `hcl.parse` | `text: string` | `any` | Parse an HCL string into a value |
| `hcl.stringify` | `value: any` | `string` | Serialize a value to HCL |

**Example file:** [`examples/hcl_example.corvo`](examples/hcl_example.corvo)

---

## `template` — Handlebars Templating

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `template.render` | `template: string, data: map` | `string` | Render a [Handlebars](https://handlebarsjs.com) template string with `data` |
| `template.render_file` | `path: string, data: map` | `string` | Load a template from `path` and render it with `data` |

**Example file:** [`examples/template_example.corvo`](examples/template_example.corvo)

---

## `llm` — Large Language Models *(placeholder)*

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `llm.model` | `name: string, [provider: string]` | `string` | Build a `"provider:model"` identifier |
| `llm.prompt` | `model: string, prompt: string` | `string` | Send a prompt and get a text response |
| `llm.embed` | `model: string, text: string` | `list[number]` | Get an embedding vector for text |
| `llm.chat` | `id: string, model: string, messages: list[map]` | `map{role, content}` | Multi-turn chat with message history |

**Example file:** [`examples/llm_example.corvo`](examples/llm_example.corvo)

---

## `notifications` — Notifications

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `notifications.smtp` | `host: string, port: number, username: string, password: string, from_addr: string, to_addr: string, subject: string, body: string` | `map{success}` | Send an email via SMTP (STARTTLS) |
| `notifications.slack` | `webhook_url: string, message: string` | `map{status_code, response_body}` | Post a message to a Slack incoming webhook |
| `notifications.telegram` | `bot_token: string, chat_id: string, message: string` | `map{status_code, response_body}` | Send a message via the Telegram Bot API |
| `notifications.mattermost` | `webhook_url: string, message: string` | `map{status_code, response_body}` | Post a message to a Mattermost incoming webhook |
| `notifications.gitter` | `token: string, room_id: string, message: string` | `map{status_code, response_body}` | Post a message to a Gitter room |
| `notifications.messenger` | `page_access_token: string, recipient_id: string, message: string` | `map{status_code, response_body}` | Send a message via the Facebook Messenger Send API |
| `notifications.discord` | `webhook_url: string, message: string` | `map{status_code, response_body}` | Post a message to a Discord webhook |
| `notifications.teams` | `webhook_url: string, message: string` | `map{status_code, response_body}` | Post a message to a Microsoft Teams incoming webhook |
| `notifications.x` | `api_key: string, api_secret: string, access_token: string, access_token_secret: string, message: string` | `map{status_code, response_body}` | Post a tweet via the Twitter/X API v2 (OAuth 1.0a) |
| `notifications.os` | `title: string, message: string` | `map{success}` | Show a local desktop notification (Linux: `notify-send`, macOS: `osascript`, Windows: PowerShell toast) |
| `notifications.irc` | `host: string, port: number, nickname: string, channel: string, message: string, [password: string]` | `map{success}` | Send a PRIVMSG to an IRC channel over plain TCP; `password` is optional (pass `""` to skip PASS) |

**Example file:** [`examples/notifications_example.corvo`](examples/notifications_example.corvo)

---

## `string` — String Type Methods

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `string.concat` | `s: string, s2: string` | `string` | Concatenate two strings |
| `string.replace` | `s: string, old: string, new: string` | `string` | Replace all occurrences of `old` with `new` |
| `string.split` | `s: string, delimiter: string` | `list[string]` | Split on a delimiter |
| `string.trim` | `s: string` | `string` | Remove leading and trailing whitespace |
| `string.contains` | `s: string, substr: string` | `bool` | Check if `s` contains `substr` |
| `string.starts_with` | `s: string, prefix: string` | `bool` | Check if `s` starts with `prefix` |
| `string.ends_with` | `s: string, suffix: string` | `bool` | Check if `s` ends with `suffix` |
| `string.to_lower` | `s: string` | `string` | Convert to lowercase |
| `string.to_upper` | `s: string` | `string` | Convert to uppercase |
| `string.len` | `s: string` | `number` | Number of characters |
| `string.reverse` | `s: string` | `string` | Reverse the characters |
| `string.is_empty` | `s: string` | `bool` | Check if the string has zero length |
| `string.pad_start` | `s: string, width: number, [fill: string]` | `string` | Pad on the left to `width` characters (default fill space) |
| `string.pad_end` | `s: string, width: number, [fill: string]` | `string` | Pad on the right to `width` characters (default fill space) |

**Example file:** [`examples/string_methods.corvo`](examples/string_methods.corvo)

---

## `number` — Number Type Methods

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `number.to_string` | `n: number` | `string` | Convert a number to its string representation |
| `number.parse` | `_: number, s: string` | `number` | Parse a string into a number |
| `number.abs` | `n: number` | `number` | Absolute value |
| `number.floor` | `n: number` | `number` | Round down to the nearest integer |
| `number.ceil` | `n: number` | `number` | Round up to the nearest integer |
| `number.round` | `n: number` | `number` | Round to the nearest integer |
| `number.sqrt` | `n: number` | `number` | Square root — error if `n < 0` |
| `number.is_nan` | `n: number` | `bool` | Check if `n` is NaN |
| `number.is_finite` | `n: number` | `bool` | Check if `n` is finite |
| `number.is_infinite` | `n: number` | `bool` | Check if `n` is infinite |

**Example file:** [`examples/number_methods.corvo`](examples/number_methods.corvo)

---

## `list` — List Type Methods

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `list.new` | _(none)_ | `list` | Create a new empty list |
| `list.push` | `l: list, item: any` | `list` | Return a new list with `item` appended |
| `list.pop` | `l: list` | `list` | Return a new list with the last item removed |
| `list.get` | `l: list, index: number` | `any` | Get item at `index` (0-based) |
| `list.set` | `l: list, index: number, value: any` | `list` | Return a new list with item at `index` replaced |
| `list.first` | `l: list` | `any` | Get the first item |
| `list.last` | `l: list` | `any` | Get the last item |
| `list.len` | `l: list` | `number` | Number of items |
| `list.is_empty` | `l: list` | `bool` | Check if the list has zero items |
| `list.contains` | `l: list, item: any` | `bool` | Check if `item` is in the list |
| `list.reverse` | `l: list` | `list` | Return a new list in reverse order |
| `list.join` | `l: list, delimiter: string` | `string` | Join all items into a string |
| `list.sort_version` | `l: list` | `list` | Sort items (`Display` / string comparison) using GNU `strverscmp`-compatible ordering |
| `list.sort_maps_by_key` | `l: list[map], key: string, [reverse: bool]` | `list[map]` | Stable sort of maps by string key (values coerced to string for comparison) |

**Example file:** [`examples/list_methods.corvo`](examples/list_methods.corvo)

---

## `map` — Map Type Methods

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `map.get` | `m: map, key: string, [default: any]` | `any` | Get value for `key` (optional default) |
| `map.set` | `m: map, key: string, value: any` | `map` | Return a new map with `key` set to `value` |
| `map.new` | _(none)_ | `map` | Create a new empty map |
| `map.remove` | `m: map, key: string` | `map` | Return a new map without `key` |
| `map.has_key` | `m: map, key: string` | `bool` | Check if `key` exists |
| `map.keys` | `m: map` | `list[string]` | Return all keys as a list |
| `map.values` | `m: map` | `list[any]` | Return all values as a list |
| `map.len` | `m: map` | `number` | Number of entries |
| `map.is_empty` | `m: map` | `bool` | Check if the map has zero entries |
| `map.merge` | `m: map, other: map` | `map` | Merge two maps (keys in `other` overwrite) |

**Example file:** [`examples/map_methods.corvo`](examples/map_methods.corvo)

---

## `var` and `static` — Variable Storage

These are language-level constructs, not stdlib functions, but are used in every program.

| Statement | Description |
|---|---|
| `var.set("name", value)` | Store a runtime variable |
| `var.get("name")` | Retrieve a runtime variable |
| `@name = value` | Shorthand for `var.set("name", value)` |
| `@name` | Shorthand for `var.get("name")` |
| `@name++` | Increment number variable by 1 |
| `@name--` | Decrement number variable by 1 |
| `@name += n` | Add `n` to a number variable |
| `@name -= n` | Subtract `n` from a number variable |
| `@name += "str"` | Concatenate `"str"` to a string variable |
| `@name -= "str"` | Remove all occurrences of `"str"` from a string variable |
| `static.set("name", value)` | Store a compile-time constant (inside `prep {}`) |
| `static.get("name")` | Retrieve a compile-time constant |

---

## Assertion Functions

Used inside `try` blocks to drive conditional branching.

| Function | Parameters | Description |
|---|---|---|
| `assert_eq` | `a: any, b: any` | Fails (triggers fallback) if `a != b` |
| `assert_neq` | `a: any, b: any` | Fails if `a == b` |
| `assert_gt` | `a: number, b: number` | Fails if `a <= b` |
| `assert_lt` | `a: number, b: number` | Fails if `a >= b` |
| `assert_match` | `a: string, pattern: string` | Fails if `a` does not match `pattern` |

**Example file:** [`examples/error_handling.corvo`](examples/error_handling.corvo)

---

## Control Flow Quick Reference

```corvo
# Match expression (if/else equivalent for value-based branching)
@result = match(@value) {
    "a" => "got a",
    "b" => "got b",
    _   => "something else"
}

# Conditional branching via try/fallback (for assertions and error handling)
try {
    assert_eq(@value, "expected")
    sys.echo("matched")
} fallback {
    sys.echo("did not match")
}

# Loop with early exit
loop {
    @counter = math.add(@counter, 1)
    try {
        assert_eq(@counter, 10)
        terminate          # break out of the loop
    } fallback {}
}

# Iteration over a list or map
browse(var.get("items"), idx, item) {
    sys.echo("${@idx}: ${@item}")
}

# Compile-time constants
prep {
    static.set("version", "1.0.0")
}
sys.echo(static.get("version"))

# Procedures — reusable callable blocks (pass-by-reference via copy-in/copy-out)
@add = procedure(@a, @b, @out) {
    @out = math.add(@a, @b)
}
@n1 = 10
@n2 = 21
@total = 0
@add.call(@n1, @n2, @total)
sys.echo(@total)   # 31
```
