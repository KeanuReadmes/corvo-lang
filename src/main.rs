use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "corvo",
    about = "Corvo Programming Language",
    after_help = "Examples:\n  corvo script.corvo              Run a file\n  corvo --repl                    Start interactive REPL\n  corvo --eval 'sys.echo(\"hi\")'  Evaluate an expression\n  corvo --compile script.corvo    Compile to standalone executable\n  corvo --check script.corvo      Check syntax"
)]
struct Args {
    #[structopt(help = "Corvo file to execute or compile")]
    file: Option<PathBuf>,

    #[structopt(short, long, help = "Start the REPL")]
    repl: bool,

    #[structopt(short, long, help = "Print version")]
    version: bool,

    #[structopt(short, long, help = "Evaluate a string")]
    eval: Option<String>,

    #[structopt(long, help = "Check syntax without executing")]
    check: bool,

    #[structopt(long, help = "Compile to standalone executable")]
    compile: bool,

    #[structopt(
        short,
        long,
        help = "Output path for compiled executable",
        parse(from_os_str)
    )]
    output: Option<PathBuf>,

    #[structopt(long, help = "Use debug build mode (faster compile)")]
    debug: bool,
}

fn main() {
    let args = Args::from_args();

    if args.version {
        println!("Corvo Language v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.repl {
        corvo_lang::run_repl();
        return;
    }

    if let Some(source) = args.eval {
        match corvo_lang::run_source(&source) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(e.exit_code());
            }
        }
        return;
    }

    if let Some(file) = args.file {
        if args.compile {
            compile_file(&file, args.output.as_deref(), args.debug);
        } else if args.check {
            check_syntax(&file);
        } else {
            run_file(&file);
        }
        return;
    }

    print_usage();
    std::process::exit(1);
}

fn run_file(file: &std::path::Path) {
    match corvo_lang::run_file(file) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(e.exit_code());
        }
    }
}

fn compile_file(file: &std::path::Path, output: Option<&std::path::Path>, debug: bool) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Cannot read '{}': {}", file.display(), e);
            std::process::exit(1);
        }
    };

    // Determine output path
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = file.file_stem().unwrap_or_default().to_string_lossy();
            if cfg!(target_os = "windows") {
                PathBuf::from(format!("{}.exe", stem))
            } else {
                PathBuf::from(stem.to_string())
            }
        }
    };

    let mut compiler = corvo_lang::compiler::Compiler::new(source, file.to_path_buf());
    if debug {
        compiler = compiler.with_debug();
    }

    eprintln!(
        "Pre-executing {} to capture static values...",
        file.display()
    );
    match compiler.pre_execute() {
        Ok(_) => {
            let statics = compiler.static_count();
            if statics > 0 {
                eprintln!("Captured {} static value(s)", statics);
            }
        }
        Err(e) => {
            eprintln!("Warning: Pre-execution error: {}", e);
        }
    }

    eprintln!("Compiling {}...", file.display());

    match compiler.compile(&output_path) {
        Ok(binary) => {
            eprintln!("Compiled successfully: {}", binary.display());
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn check_syntax(file: &std::path::Path) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Cannot read '{}': {}", file.display(), e);
            std::process::exit(1);
        }
    };

    match corvo_lang::lexer::Lexer::new(&source).tokenize() {
        Ok(tokens) => match corvo_lang::parser::Parser::new(tokens).parse() {
            Ok(_) => println!("Syntax OK: {}", file.display()),
            Err(e) => {
                eprintln!("Syntax Error: {}", e);
                std::process::exit(e.exit_code());
            }
        },
        Err(e) => {
            eprintln!("Lex Error: {}", e);
            std::process::exit(e.exit_code());
        }
    }
}

fn print_usage() {
    eprintln!("Usage: corvo [OPTIONS] [FILE]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -r, --repl           Start the REPL");
    eprintln!("  -e, --eval <EXPR>    Evaluate an expression");
    eprintln!("  -c, --compile        Compile to standalone executable");
    eprintln!("  -o, --output <PATH>  Output path (for --compile)");
    eprintln!("  -v, --version        Print version");
    eprintln!("      --check          Check syntax without executing");
    eprintln!("      --debug          Use debug build mode (faster compile)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  corvo script.corvo               Run a file");
    eprintln!("  corvo --repl                     Start interactive REPL");
    eprintln!("  corvo --compile script.corvo     Compile to executable");
    eprintln!("  corvo --compile script.corvo -o myapp");
    eprintln!("  corvo --eval 'sys.echo(\"hi\")'   Evaluate an expression");
}
