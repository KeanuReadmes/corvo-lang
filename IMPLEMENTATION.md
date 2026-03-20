# Corvo Language Reference (AI Assistant Context)

## 1. AI Generation Directives
When generating Corvo code, you **must** strictly adhere to the following language rules:
* **NO USER-DEFINED FUNCTIONS:** Do not generate `def`, `fn`, `function`, or lambda expressions. All operations must use the built-in library calls.
* **NO IF/ELSE STATEMENTS:** Do not generate `if`, `elif`, `else`, `switch`, or `match`. Control flow is strictly handled via `try { ... } fallback { ... }` combined with `assert_*` commands.
* **NO ASSIGNMENT OPERATORS:** Do not use `=` for assignment. State is strictly managed via `var.set()` and `static.set()`.
* **STRING INTERPOLATION:** Use `${}` for string interpolation (e.g., `sys.echo("Value: ${var.get("key")}")`).
* **NAMED PARAMETERS:** Library functions support Python-like named parameters for clarity (e.g., `http.get(url: "https://...")`).
* **IMMUTABILITY OF METHODS:** Type methods (like `string.replace`) do not mutate the variable in place; they return a new value that must be reassigned via `var.set()`.

## 2. Type System & Type Methods
Corvo is strongly and limited typed. A variable's type is inferred on its first assignment and cannot change. Type methods are called using the type namespace (e.g., `string.method(value, ...)`).

### `string`
* `string.concat(str1: string, str2: string) -> string`
* `string.replace(target: string, old: string, new: string) -> string`
* `string.split(target: string, delimiter: string) -> list`
* `string.trim(target: string) -> string`
* `string.contains(target: string, substr: string) -> boolean`
* `string.starts_with(target: string, prefix: string) -> boolean`
* `string.ends_with(target: string, suffix: string) -> boolean`
* `string.to_lower(target: string) -> string`
* `string.to_upper(target: string) -> string`
* `string.len(target: string) -> number`
* `string.reverse(target: string) -> string`
* `string.is_empty(target: string) -> boolean`

### `number`
* `number.to_string(num: number) -> string`
* `number.parse(str: string) -> number` (Fails if invalid, triggering fallback)
* `number.is_nan(num: number) -> boolean`

### `list`
* `list.push(target: list, item: any) -> list` (Returns a new list with the item appended)
* `list.pop(target: list) -> list` (Returns a new list without the last item)
* `list.get(target: list, index: number) -> any`
* `list.set(target: list, index: number, value: any) -> list` (Returns a new list with the item at index replaced)
* `list.first(target: list) -> any`
* `list.last(target: list) -> any`
* `list.len(target: list) -> number`
* `list.is_empty(target: list) -> boolean`
* `list.contains(target: list, item: any) -> boolean`
* `list.reverse(target: list) -> list`
* `list.join(target: list, delimiter: string) -> string`

### `map`
* `map.keys(target: map) -> list`
* `map.values(target: map) -> list`
* `map.len(target: map) -> number`
* `map.is_empty(target: map) -> boolean`
* `map.has_key(target: map, key: string) -> boolean`
* `map.get(target: map, key: string, default: any) -> any`
* `map.set(target: map, key: string, value: any) -> map` (Returns a new map with the updated key)
* `map.remove(target: map, key: string) -> map` (Returns a new map without the specified key)
* `map.merge(target: map, other: map) -> map` (Returns a new map with keys from both maps)

## 3. State Management

### 3.1 Runtime Variables (`var`)
Used for dynamic data that changes during execution.
```corvo
var.set("target_dir", "/var/www/html")
# Type method usage
var.set("target_dir", string.concat(var.get("target_dir"), "/public"))
```

### 3.2 Compile-Time Constants (`static`)
Used for configuration and immutable values. These are encrypted and baked into the compiled binary.
```corvo
static.set("appName", "Colonia_Agent")
static.set("db_password", os.get_env("DB_PASS", "default_secret"))
```

## 4. Control Flow

### 4.1 Try / Fallback (Ifless Architecture)
Conditionals and error handling are unified. Execution proceeds linearly until an `assert_*` fails or a command errors out, which instantly triggers the next `fallback` block.

**Built-in Assertions:**
* `assert_eq(a, b)`: Asserts `a` equals `b`.
* `assert_neq(a, b)`: Asserts `a` does not equal `b`.
* `assert_gt(a, b)`: Asserts `a` > `b`.
* `assert_lt(a, b)`: Asserts `a` < `b`.
* `assert_match(regex: string, target: string)`: Asserts string matches regex.

### 4.2 Loops (`loop` & `terminate`)
Corvo supports a single, infinite loop construct. The only way to exit is by calling `terminate`.

## 5. Comprehensive Standard Library

*(Note: All libraries support the `?` operator for in-code/REPL documentation, e.g., `fs.read?`)*

### Core & System (`sys`, `os`, `math`)
* `sys.echo(msg: string)`: Prints to stdout.
* `sys.read_line(prompt: string) -> string`: Reads user input from stdin.
* `sys.sleep(ms: number)`: Pauses execution.
* `sys.panic(msg: string)`: Terminates with a non-zero exit code.
* `sys.exec(cmd: list, input?: string, check?: boolean, timeout?: number, cwd?: string, env?: map) -> map`: Executes a process directly without a shell. The first argument is a list of strings where the first element is the program and the remaining elements are its arguments (e.g., `["ls", "-la", "/tmp"]`). Returns `{"stdout": string, "stderr": string, "code": number}`. Named parameters: `input` (data piped to stdin), `check` (error on non-zero exit), `timeout` (kill after N seconds, triggers fallback on timeout), `cwd` (working directory), `env` (environment variables). Use `sys.exec` when you need direct process invocation, piping, timeouts, or environment control.
* `os.get_env(key: string, default: string) -> string`
* `os.set_env(key: string, value: string)`
* `os.exec(cmd: string, args: list) -> map`: Simple process execution without a shell. Returns `{"stdout": string, "stderr": string, "code": number}`. Use `os.exec` for direct process invocation when you have a command and its arguments as separate values and do not need shell features.
* `os.info() -> map`: Returns `{"os": string, "arch": string, "hostname": string}`.
* `math.add(a: number, b: number) -> number`
* `math.sub(a: number, b: number) -> number`
* `math.mul(a: number, b: number) -> number`
* `math.div(a: number, b: number) -> number`
* `math.mod(a: number, b: number) -> number`

### Hashing & Encryption (`crypto`)
* `crypto.hash(algorithm: string, data: string) -> string` (Supports "md5", "sha256", "sha512")
* `crypto.encrypt(data: string, key: string) -> string` (AES-GCM)
* `crypto.decrypt(data: string, key: string) -> string`
* `crypto.uuid() -> string` (Generates a UUIDv4)

### File System (`fs`)
* `fs.read(path: string) -> string`
* `fs.write(path: string, content: string) -> boolean`
* `fs.append(path: string, content: string) -> boolean`
* `fs.delete(path: string) -> boolean`
* `fs.exists(path: string) -> boolean`
* `fs.mkdir(path: string, recursive: boolean) -> boolean`
* `fs.list_dir(path: string) -> list`
* `fs.copy(src: string, dest: string) -> boolean`
* `fs.move(src: string, dest: string) -> boolean`
* `fs.stat(path: string) -> map`: Returns `{"size": number, "is_dir": boolean, "permissions": string, "modified_at": number}`.

### Networking & Web (`http`, `dns`)
* `http.get(url: string, headers: map) -> map`: Returns `{"status_code": number, "response_body": string, "headers": map}`.
* `http.post(url: string, body: string, headers: map) -> map`
* `http.put(url: string, body: string, headers: map) -> map`
* `http.delete(url: string, headers: map) -> map`
* `dns.resolve(hostname: string) -> list`: Returns list of IP addresses.
* `dns.lookup(ip: string) -> string`: Returns hostname.

### Remote Operations (`ssh`, `rsync`)
* `ssh.exec(host: string, user: string, key_path: string, cmd: string) -> map`: Returns `{"stdout": string, "stderr": string, "code": number}`.
* `ssh.scp_upload(host: string, user: string, key_path: string, local_path: string, remote_path: string) -> boolean`
* `ssh.scp_download(host: string, user: string, key_path: string, remote_path: string, local_path: string) -> boolean`
* `rsync.sync(source: string, dest: string, options: list) -> boolean`

### Data Parsing (`json`, `yaml`, `hcl`, `csv`, `xml`)
* `json.parse(data: string) -> map`
* `json.stringify(data: map) -> string`
* `yaml.parse(data: string) -> map`
* `yaml.stringify(data: map) -> string`
* `hcl.parse(data: string) -> map` (Essential for parsing Terraform configs)
* `hcl.stringify(data: map) -> string`
* `csv.parse(data: string, delimiter: string) -> list`
* `xml.parse(data: string) -> map`

### Security & Crypto (`crypto`)
* `crypto.hash(algorithm: string, data: string) -> string` (Supports "md5", "sha256", "sha512")
* `crypto.encrypt(data: string, key: string) -> string` (AES-GCM)
* `crypto.decrypt(data: string, key: string) -> string`
* `crypto.uuid() -> string` (Generates a UUIDv4)

### Artificial Intelligence (`llm`)
* `llm.model(name: string, provider: string, options: map) -> string`: Tests and build a connection string for the other llm functions. The provider can be "openai", "gemini", "azure", "local (llama.cpp)", "ollama". The options map can include parameters like API keys, model versions, etc.
* `llm.prompt(model: string, prompt: string, tokens: number) -> string`: Executes a prompt against the specified model (the connection string returned by `llm.model`) and returns the generated text.
* `llm.embed(model: string, text: string) -> list`: Returns a vector embedding for the given text using the specified model.
* `llm.chat(id: string, model: string, messages: list, tokens: number) -> map`: Executes a chat conversation with the specified model. The `messages` parameter is a list of maps, each containing `{"role": "user|assistant|system", "content": string}`. The function returns the assistant's response.
