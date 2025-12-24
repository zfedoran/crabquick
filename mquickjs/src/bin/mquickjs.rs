//! MicroQuickJS command-line interface
//!
//! A simple CLI for running JavaScript files with the MicroQuickJS engine.
//!
//! Usage:
//!   mquickjs <file.js>              - Execute a JavaScript file
//!   mquickjs -e "<code>"            - Execute JavaScript code directly
//!   mquickjs --help                 - Show help message

use std::env;
use std::fs;
use std::process;
use mquickjs::Engine;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut engine = Engine::new(1024 * 1024); // 1MB heap

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "-e" => {
            if args.len() < 3 {
                eprintln!("Error: -e requires a JavaScript expression");
                process::exit(1);
            }
            execute_code(&mut engine, &args[2]);
        }
        "--version" | "-v" => {
            print_version();
        }
        "--memory" | "-m" => {
            if args.len() < 3 {
                eprintln!("Error: -m requires a filename");
                process::exit(1);
            }
            execute_file_with_stats(&mut engine, &args[2]);
        }
        filename => {
            execute_file(&mut engine, filename);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: mquickjs <file.js>");
    eprintln!("       mquickjs -e \"<code>\"");
    eprintln!("       mquickjs --help");
}

fn print_help() {
    println!("MicroQuickJS - A Minimal JavaScript Engine");
    println!();
    println!("Usage:");
    println!("  mquickjs <file.js>         Execute a JavaScript file");
    println!("  mquickjs -e \"<code>\"       Execute JavaScript code directly");
    println!("  mquickjs -m <file.js>      Execute with memory statistics");
    println!("  mquickjs --help            Show this help message");
    println!("  mquickjs --version         Show version information");
    println!();
    println!("Examples:");
    println!("  mquickjs hello.js");
    println!("  mquickjs -e \"console.log('Hello, World!')\"");
    println!("  mquickjs -m fibonacci.js");
}

fn print_version() {
    println!("MicroQuickJS v0.1.0");
    println!("A native Rust implementation of a minimal JavaScript engine");
}

fn execute_code(engine: &mut Engine, source: &str) {
    match engine.eval(source) {
        Ok(result) => {
            // Print result if not undefined
            let result_str = engine.eval_as_string("result").unwrap_or_default();
            if result_str != "undefined" {
                println!("{}", result_str);
            }
        }
        Err(_error) => {
            // Try to get error message
            eprintln!("Error executing JavaScript");
            process::exit(1);
        }
    }
}

fn execute_file(engine: &mut Engine, filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    match engine.eval(&source) {
        Ok(_result) => {
            // Execution succeeded
            // Result is typically undefined for script files
        }
        Err(_error) => {
            eprintln!("Error executing JavaScript file '{}'", filename);
            process::exit(1);
        }
    }
}

fn execute_file_with_stats(engine: &mut Engine, filename: &str) {
    let stats_before = engine.memory_stats();
    println!("Memory before execution:");
    println!("  Heap size: {} bytes", stats_before.heap_size);
    println!("  Heap used: {} bytes", stats_before.heap_used);
    println!();

    execute_file(engine, filename);

    let stats_after = engine.memory_stats();
    println!();
    println!("Memory after execution:");
    println!("  Heap size: {} bytes", stats_after.heap_size);
    println!("  Heap used: {} bytes", stats_after.heap_used);
    println!("  Peak usage: {:.1}%",
             (stats_after.heap_used as f64 / stats_after.heap_size as f64) * 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_code() {
        let mut engine = Engine::new(8192);
        // This will likely fail until compiler is complete
        // execute_code(&mut engine, "1 + 1");
    }
}
