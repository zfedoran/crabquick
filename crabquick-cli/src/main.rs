//! CrabQuick command-line interface

use crabquick::Engine;

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    // Handle different command-line options
    match args[1].as_str() {
        "--help" | "-h" => {
            print_usage();
            std::process::exit(0);
        }
        "--version" | "-v" => {
            println!("CrabQuick v{}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        "-e" | "--eval" => {
            if args.len() < 3 {
                eprintln!("Error: -e requires a script argument");
                print_usage();
                std::process::exit(1);
            }
            eval_script(&args[2]);
        }
        "--repl" => {
            eprintln!("REPL mode not yet implemented");
            std::process::exit(1);
        }
        filename => {
            // Try to read and execute a script file
            match std::fs::read_to_string(filename) {
                Ok(source) => eval_script(&source),
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", filename, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_usage() {
    println!("CrabQuick v{} - A minimal JavaScript engine", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage:");
    println!("  crabquick -e <script>     Evaluate JavaScript code");
    println!("  crabquick <script.js>     Execute JavaScript file");
    println!("  crabquick --repl          Start interactive REPL (not yet implemented)");
    println!("  crabquick --help          Show this help message");
    println!("  crabquick --version       Show version information");
    println!();
    println!("Examples:");
    println!("  crabquick -e \"1 + 2\"");
    println!("  crabquick -e \"console.log('hello')\"");
    println!("  crabquick script.js");
}

fn eval_script(source: &str) {
    // Create engine with 64 KB memory (enough for most scripts)
    let mut engine = Engine::new(65536);

    // Execute the script
    match engine.eval_as_string(source) {
        Ok(result) => {
            // Only print non-undefined results
            if result != "undefined" {
                println!("{}", result);
            }
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}
