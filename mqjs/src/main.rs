//! MicroQuickJS command-line interface

use mquickjs::Context;

fn main() {
    println!("MicroQuickJS v{}", env!("CARGO_PKG_VERSION"));

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: mqjs [options] <script.js>");
        println!("       mqjs --repl");
        std::process::exit(1);
    }

    // Create context with 8 KB memory
    let mut ctx = Context::new(8192);

    // TODO: Implement script execution
    // TODO: Implement REPL mode

    println!("Execution complete. Memory usage: {} bytes", ctx.memory_usage());
}
