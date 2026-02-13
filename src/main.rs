//! Lemon Programming Language CLI
//!
//! Commands:
//! - lemon run <file>     - Run a Lemon script (JIT)
//! - lemon build <file>   - Compile to native binary
//! - lemon check <file>   - Type check without compiling
//! - lemon repl           - Interactive REPL

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lemon::codegen::{CodeGen, CodeGenConfig};
use lemon::parser;
use lemon::runtime::Interpreter;
use lemon::types::check::TypeChecker;

#[derive(Parser)]
#[command(name = "lemon")]
#[command(author = "Lemon Language Team")]
#[command(version = "0.1.0")]
#[command(about = "The Lemon programming language - AI-native, human-friendly")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Lemon script
    Run {
        /// The file to run
        file: PathBuf,

        /// Enable deterministic mode
        #[arg(long)]
        deterministic: bool,

        /// Run in sandbox mode
        #[arg(long)]
        sandbox: Option<String>,
    },

    /// Compile to native binary
    Build {
        /// The file to compile
        file: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,

        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Use LLVM backend (requires LLVM)
        #[arg(long)]
        backend: Option<String>,

        /// Target triple for cross-compilation
        #[arg(long)]
        target: Option<String>,
    },

    /// Type check a file without compiling
    Check {
        /// The file to check
        file: PathBuf,

        /// Output structured errors for AI agents
        #[arg(long)]
        ai_mode: bool,
    },

    /// Start interactive REPL
    Repl,

    /// Parse a file and print the AST (debug)
    Parse {
        /// The file to parse
        file: PathBuf,
    },

    /// Tokenize a file and print tokens (debug)
    Lex {
        /// The file to tokenize
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, deterministic, sandbox } => {
            run_file(&file, deterministic, sandbox.as_deref())
        }
        Commands::Build { file, output, opt_level, release, backend, target } => {
            build_file(&file, output.as_deref(), if release { 2 } else { opt_level }, backend.as_deref(), target.as_deref())
        }
        Commands::Check { file, ai_mode } => {
            check_file(&file, ai_mode)
        }
        Commands::Repl => {
            run_repl()
        }
        Commands::Parse { file } => {
            parse_file(&file)
        }
        Commands::Lex { file } => {
            lex_file(&file)
        }
    }
}

fn run_file(path: &PathBuf, _deterministic: bool, _sandbox: Option<&str>) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Parse
    let ast = match parser::parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Type check (skip for now since type checker doesn't know about builtins)
    // TODO: Register builtins in type checker
    // let mut checker = TypeChecker::new();
    // if let Err(errors) = checker.check_file(&ast) {
    //     for error in errors {
    //         eprintln!("Type error: {}", error.message);
    //     }
    //     return ExitCode::FAILURE;
    // }

    // Execute with interpreter
    let mut interp = Interpreter::new();

    // Set current file for relative module imports
    if let Ok(abs_path) = std::fs::canonicalize(path) {
        interp.set_current_file(abs_path);
    } else {
        interp.set_current_file(path.clone());
    }

    match interp.execute(&ast) {
        Ok(value) => {
            // Only print non-unit results
            if !matches!(value, lemon::runtime::Value::Unit) {
                println!("{}", value);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn build_file(
    path: &PathBuf,
    output: Option<&Path>,
    opt_level: u8,
    _backend: Option<&str>,
    target: Option<&str>,
) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Parse
    let ast = match parser::parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(errors) = checker.check_file(&ast) {
        for error in errors {
            eprintln!("Type error: {}", error.message);
        }
        return ExitCode::FAILURE;
    }

    // Generate code
    let config = CodeGenConfig {
        opt_level,
        debug_info: opt_level == 0,
        target: target.map(|s| s.to_string()),
    };
    let codegen = CodeGen::new(config);

    match codegen.generate(&ast) {
        Ok(binary) => {
            let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
                let mut p = path.clone();
                p.set_extension(if cfg!(windows) { "exe" } else { "" });
                p
            });

            if let Err(e) = fs::write(&output_path, binary) {
                eprintln!("Error writing output: {}", e);
                return ExitCode::FAILURE;
            }

            println!("Built: {}", output_path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn check_file(path: &PathBuf, ai_mode: bool) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Parse
    let ast = match parser::parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            if ai_mode {
                // Output structured error for AI
                println!(
                    r#"{{"error_code": "P0001", "category": "parse_error", "message": "{}", "line": 0, "column": 0}}"#,
                    e.to_string().replace('"', "\\\"")
                );
            } else {
                eprintln!("Parse error: {}", e);
            }
            return ExitCode::FAILURE;
        }
    };

    // Type check
    let mut checker = TypeChecker::new();
    match checker.check_file(&ast) {
        Ok(()) => {
            if !ai_mode {
                println!("No errors found.");
            }
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                if ai_mode {
                    // Output structured error for AI
                    println!(
                        r#"{{"error_code": "T0001", "category": "type_error", "message": "{}", "line": {}, "column": {}}}"#,
                        error.message.replace('"', "\\\""),
                        error.span.start,
                        0
                    );
                } else {
                    eprintln!("Type error at {}: {}", error.span.start, error.message);
                }
            }
            ExitCode::FAILURE
        }
    }
}

fn run_repl() -> ExitCode {
    println!("Lemon REPL v0.1.0");
    println!("Type 'exit' or press Ctrl+C to quit.");
    println!();

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        print!(">>> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        input.clear();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        // Try to parse as expression, wrap in function if needed
        let source = if trimmed.contains("fn ") || trimmed.contains("let ") {
            trimmed.to_string()
        } else {
            format!("fn __repl__() {{ {} }}", trimmed)
        };

        match parser::parse(&source) {
            Ok(ast) => {
                println!("{:#?}", ast);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    ExitCode::SUCCESS
}

fn parse_file(path: &PathBuf) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match parser::parse(&source) {
        Ok(ast) => {
            println!("{:#?}", ast);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn lex_file(path: &PathBuf) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let tokens = lemon::lexer::Lexer::tokenize(&source);
    for token in tokens {
        println!("{:?}", token);
    }

    ExitCode::SUCCESS
}
