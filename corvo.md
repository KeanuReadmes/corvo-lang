# Corvo Language 

Corvo is a programming language designed for simplicity and readability. It emphasizes clean syntax and ease of use, making it an ideal choice for batch programming and scripting tasks. The main goal of Corvo is be a modern alternative to Shell scripting/Coreutils, providing all the necessary functionality while maintaining a user-friendly syntax.

## Key Features

* Functionless: Built-in functions only. Corvo does not support function creation, which encourages a more straightforward and linear programming style. This design choice simplifies the language and reduces the learning curve for new programmers.
* ifless: Corvo uses an ifless syntax for conditional statements, which eliminates the need for traditional if/else structures. Instead, developers can use a more concise and intuitive approach to handle conditions, improving code readability.
* Try/Fallback: Corvo includes a built-in try/fallback mechanism that allows developers to handle errors gracefully. This feature enables programmers to specify alternative actions when an error occurs, improving the robustness of scripts. It uses asset() to attempt an operation and fallback() to specify the alternative action if the operation fails.
* Built-in Commands: Corvo comes with a comprehensive set of built-in commands that cover a wide range of common programming tasks. These commands are designed to be intuitive and easy to use, allowing developers to accomplish tasks quickly and efficiently.
* Cross-Platform Compatibility: Corvo is designed to be cross-platform, allowing developers to write scripts that can run on various operating systems without modification. This makes it a versatile choice for developers who need to work across different environments.
* REPL Support: Corvo includes a Read-Eval-Print Loop (REPL) that allows developers to interactively test and debug their code. This feature provides immediate feedback and helps developers quickly identify and fix issues in their scripts.
* Compiled Language: Corvo is a compiled language, which means that scripts are compiled into executable files before they are run. This approach can improve performance and allows for better error checking during the development process.
* Strongly and limited Typed: Corvo is a strongly typed language, which means that variables must be declared with a specific type and cannot be changed to a different type. This design choice helps prevent type-related errors and promotes better code organization. Valid types: string, number, boolean, list, map.
* Static Variables: Stores immutable values for compile-time constants and configuration settings. Static variables must be defined inside a `prep` block at the top of the script and cannot be modified during runtime, ensuring that critical values remain consistent throughout the execution of the program.

## Language Syntax
Corvo's syntax is designed to be clean and straightforward. Here are some examples of Corvo code:

```corvo
# This is a comment in Corvo

# prep block — runs at compile time only.
# This is the only place where static.set() is allowed.
# Variables created here are NOT available outside the block.
prep {
    static.set("appName", "Corvo") # Compile-time constant, baked into the binary
    static.set("version", os.get_env("CORVO_VERSION")) # Read from environment at compile time
    static.set("pi", 3.14159)
}

# Print a message to the console
sys.echo("Hello, World!")
sys.echo("App: ${static.get("appName")}")

# Variable assignment
var.set("greeting", "Hello, Corvo!")
sys.echo(var.get("greeting"))

# @ shorthand: @name reads a variable (var.get), @name = val writes it (var.set)
@greeting = "Hello, shorthand!"
sys.echo(@greeting)

sys.echo(static.get("pi"))

# Try/Fallback example
try {
    sys.echo("Check if pi is greater than 3.5")
    assert_gt(static.get("pi"), 3.5)
} fallback {
    sys.echo("Pi is not greater than 3.5, checking if it's less than 3")
    assert_lt(static.get("pi"), 3)
} fallback {
    sys.echo("Pi is exactly ${static.get("pi")}")
}

# Infinite loop example, exit on `terminate` call.
loop {
    var.set("input", sys.read_line("Enter a command: "))
    try {
        assert_eq(var.get("input"), "exit")
        sys.echo("Exiting the loop. Goodbye!")
        terminate
    } fallback {
        sys.echo("You entered: ${var.get("input")}")
    }
}

# Browse block — iterate over a list or map.
# For a list: key is the zero-based index, value is the element.
# For a map:  key is the string key, value is the associated value.
# Inside the block, use the $ prefix to access browse-bound names.
var.set("fruits", ["apple", "banana", "cherry"])
browse(var.get("fruits"), idx, fruit) {
    sys.echo("${$idx}: ${$fruit}")
}
# Prints:
# 0: apple
# 1: banana
# 2: cherry

var.set("config", {"host": "localhost", "port": 8080})
browse(var.get("config"), key, val) {
    sys.echo("${$key} = ${$val}")
}
# Prints:
# host = localhost
# port = 8080
```
In this example, we demonstrate various features of Corvo, including variable assignment, static variables, and the try/fallback mechanism. The code is designed to be easy to read and understand, making it accessible for both beginners and experienced programmers.    

## Libraries

The main target of Corvo is to have all the corresponding functionality of Shell scripting/Coreutils, DNS utils, Curl, SSH, OpenSSL, Rsync, LLama.cpp, parsers for JSON, XML, CSV, DOC, DOCX, HCL, and more. The library sintax must not be a direct translation of the inspired tools, but rather a more intuitive and user-friendly approach that fits the overall design philosophy of Corvo. 
The function calls must be based on Python's syntax, which can use named parameters and a more natural way of expressing operations. This design choice allows developers to write code that is easier to read and understand, while still providing the powerful functionality needed for various programming tasks.
For example, instead of using a direct translation of the `curl` command, Corvo might provide a more intuitive syntax for making HTTP requests, such as:

```corvo
# http.get(url: string) -> response: map

var.set("req", http.get(url: "https://api.example.com/data")) # Also could be http.get("https://api.example.com/data") if the parameters are not named.
try {
    assert_eq(var.get("req")["status_code"], 200)
    sys.echo("Response: ${var.get("req")["response_body"]}")
} fallback {
    sys.echo("Failed to fetch data from the API.")
}
```

All functions in the libraries must support the `?` which prints the documentation of the function, including its parameters, return values, and examples of usage. This feature allows developers to quickly access information about the functions they are using without needing to refer to external documentation.

```corvo
# Example of using the `?` to access documentation
http.get? # Prints the documentation for the http.get function, including its parameters, return values, and examples of usage.
http? # Prints the documentation for the entire http library, including all available functions, their parameters, return values, and examples of usage.
```

Corvo must be written in Rust, leveraging its performance and safety features to create a robust and efficient programming language. The choice of Rust also allows for easy cross-platform compatibility, making Corvo accessible to a wide range of developers across different operating systems.
