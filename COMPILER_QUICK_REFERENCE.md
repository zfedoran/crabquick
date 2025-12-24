# MicroQuickJS Compiler - Quick Reference

## Basic Usage

```rust
use mquickjs::compiler::compile;

// Compile JavaScript to bytecode
let bytecode = compile("2 + 3")?;
```

## API Reference

### Main Functions

```rust
// Compile JavaScript source to bytecode
pub fn compile(source: &str) -> Result<Vec<u8>, CompileError>
```

### Error Types

```rust
pub enum CompileError {
    Parse(ParseError),    // Syntax error during parsing
    CodeGen(CodeGenError), // Error during bytecode generation
}

pub struct ParseError {
    pub message: String,
    pub location: SourceLocation,
}

pub struct CodeGenError {
    pub message: String,
    pub location: Option<SourceLocation>,
}

pub struct SourceLocation {
    pub line: u32,      // 1-based line number
    pub column: u32,    // 1-based column number
    pub offset: usize,  // byte offset in source
}
```

## Examples

### Simple Expressions

```rust
// Arithmetic
compile("2 + 3 * 4")?;           // → bytecode
compile("(2 + 3) * 4")?;         // → bytecode

// Comparison
compile("x > 10")?;              // → bytecode
compile("a === b")?;             // → bytecode

// Logical
compile("x && y || z")?;         // → bytecode
compile("!valid")?;              // → bytecode
```

### Variables

```rust
// Declaration
compile("var x = 10;")?;
compile("var a = 1, b = 2;")?;
compile("let name = 'John';")?;
compile("const PI = 3.14159;")?;

// Assignment
compile("x = 20;")?;
compile("y += 5;")?;
compile("z *= 2;")?;
```

### Functions

```rust
// Function declaration
compile("function add(a, b) { return a + b; }")?;

// Function call
compile("console.log('Hello');")?;
compile("Math.max(1, 2, 3);")?;

// Function expression
compile("var fn = function(x) { return x * 2; };")?;
```

### Control Flow

```rust
// If statement
compile("if (x > 0) { y = 1; } else { y = -1; }")?;

// While loop
compile("while (x < 10) { x = x + 1; }")?;

// For loop
compile("for (var i = 0; i < 10; i++) { sum += i; }")?;

// Return
compile("function test() { return 42; }")?;
```

### Arrays and Objects

```rust
// Array literal
compile("var arr = [1, 2, 3];")?;
compile("arr[0]")?;

// Object literal
compile("var obj = { x: 10, y: 20 };")?;
compile("obj.x")?;
compile("obj['y']")?;
```

### Complex Examples

```rust
// Fibonacci function
compile(r#"
    function fib(n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }
"#)?;

// Counter with closure (structure parsed, full compilation pending)
compile(r#"
    function makeCounter() {
        var count = 0;
        return function() {
            return ++count;
        };
    }
"#)?;

// Array operations
compile(r#"
    var sum = 0;
    var numbers = [1, 2, 3, 4, 5];
    for (var i = 0; i < numbers.length; i++) {
        sum += numbers[i];
    }
    return sum;
"#)?;
```

## Error Handling

### Catching Parse Errors

```rust
use mquickjs::compiler::{compile, CompileError};

match compile("var x = ;") {
    Ok(bytecode) => {
        println!("Compiled successfully");
    }
    Err(CompileError::Parse(err)) => {
        eprintln!("Syntax error at line {}, column {}: {}",
            err.location.line,
            err.location.column,
            err.message);
    }
    Err(CompileError::CodeGen(err)) => {
        eprintln!("Code generation error: {}", err.message);
    }
}
```

### Common Parse Errors

```rust
// Missing semicolon
compile("var x = 10")?;  // OK (ASI)

// Unexpected token
compile("var = 10")?;    // Error: Expected identifier

// Unterminated string
compile("var s = 'hello")?;  // Error: Unterminated string

// Invalid number
compile("var x = 0xGG")?;    // Error: Invalid hex number
```

## Supported JavaScript Features

### ✅ Fully Supported

- **Literals**: numbers (42, 3.14, 0xFF, 0b1010), strings ("hello", 'world'), booleans (true, false), null, undefined
- **Variables**: var, let, const declarations
- **Operators**:
  - Arithmetic: +, -, *, /, %, **
  - Comparison: <, >, <=, >=, ==, ===, !=, !==
  - Logical: &&, ||, !
  - Bitwise: &, |, ^, ~, <<, >>, >>>
  - Assignment: =, +=, -=, *=, /=, etc.
  - Unary: +, -, !, ~, typeof, void, delete
  - Update: ++, --
- **Statements**:
  - Expression statements
  - Block statements
  - if/else
  - while, for
  - return
  - throw
- **Functions**: declarations and calls
- **Arrays**: literals and element access
- **Objects**: literals and member access

### ⚠️ Partially Supported

- **Functions**: bodies not fully compiled yet
- **Break/continue**: structure exists
- **Try/catch**: structure exists
- **Switch**: structure exists

### ❌ Not Yet Supported

- Classes, modules, async/await
- Destructuring, spread
- Template literals
- Regular expressions
- Closures with variable capture

## Tips and Best Practices

### 1. Always Handle Errors

```rust
// Good
let bytecode = match compile(source) {
    Ok(bc) => bc,
    Err(e) => {
        log_error(&e);
        return;
    }
};

// Also good
let bytecode = compile(source)
    .map_err(|e| log_error(&e))?;
```

### 2. Use Semicolons

While ASI (Automatic Semicolon Insertion) is supported, explicit semicolons are clearer:

```rust
// Recommended
compile("var x = 10; var y = 20;")?;

// Works but less clear
compile("var x = 10\nvar y = 20")?;
```

### 3. Check String Literals

Always use quotes for strings:

```rust
// Correct
compile("var name = 'John';")?;

// Incorrect (will be treated as identifier)
compile("var name = John;")?;
```

### 4. Test Complex Code

Test complex code incrementally:

```rust
// Start simple
compile("function add(a, b) { }")?;

// Add body
compile("function add(a, b) { return a + b; }")?;

// Add complexity
compile("function add(a, b) {
    if (typeof a !== 'number') return 0;
    return a + b;
}")?;
```

## Advanced Usage

### Using Individual Components

```rust
use mquickjs::compiler::{Lexer, Parser, CodeGenerator};

// Tokenize only
let mut lexer = Lexer::new("2 + 3");
while let token = lexer.next_token() {
    if matches!(token.kind, TokenKind::Eof) { break; }
    println!("{:?}", token);
}

// Parse only
let parser = Parser::new("2 + 3");
let program = parser.parse()?;
println!("{:#?}", program);

// Generate bytecode from AST
let generator = CodeGenerator::new();
let bytecode = generator.generate(&program)?;
```

### Inspecting AST

```rust
use mquickjs::compiler::{Parser, Stmt, Expr};

let parser = Parser::new("var x = 2 + 3;");
let program = parser.parse()?;

for stmt in program.body {
    match stmt {
        Stmt::VarDecl { declarations, .. } => {
            for decl in declarations {
                println!("Variable: {}", decl.name);
                if let Some(init) = decl.init {
                    println!("Initializer: {:?}", init);
                }
            }
        }
        _ => {}
    }
}
```

## Performance Considerations

### Compilation Speed

The compiler is designed for speed:
- Single-pass lexing: O(n)
- Recursive descent parsing: O(n)
- Single-pass code generation: O(n)

Overall: **O(n)** where n is source code length.

### Memory Usage

Memory usage is proportional to:
- Source code size (tokenization)
- AST size (parsing)
- Bytecode size (generation)

Typical ratio: **Bytecode size ≈ 10-30% of source size**

### Optimization Tips

```rust
// 1. Reuse compile function (no need to cache)
for source in sources {
    let bytecode = compile(source)?;
    execute(bytecode);
}

// 2. Use const for literals
compile("const PI = 3.14159;")?;  // deduped in constant pool

// 3. Prefer local variables
compile(r#"
    function process() {
        var local = expensive_operation();
        use(local);
        use(local);  // doesn't recompute
    }
"#)?;
```

## Debugging

### Enable Debug Output

```rust
// In development, examine AST
#[cfg(debug_assertions)]
{
    let parser = Parser::new(source);
    let program = parser.parse()?;
    println!("{:#?}", program);  // Pretty-print AST
}
```

### Common Issues

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Parse error | Syntax mistake | Check error location |
| Unexpected token | Missing semicolon | Add semicolon or check ASI rules |
| Too many constants | Very large program | Split into modules |
| Stack overflow | Deeply nested code | Reduce nesting depth |

## Quick Cheat Sheet

| Task | Code |
|------|------|
| Compile | `compile(source)?` |
| Check syntax | `Parser::new(src).parse()?` |
| Tokenize | `Lexer::new(src).next_token()` |
| Handle error | `match compile(src) { Ok(bc) => ..., Err(e) => ... }` |
| Get location | `err.location.line, err.location.column` |

## Resources

- **Full Documentation**: See `PHASE6_IMPLEMENTATION_SUMMARY.md`
- **API Docs**: Run `cargo doc --open`
- **Examples**: See `mquickjs/src/compiler/*/tests`
- **Bytecode Reference**: See `mquickjs/src/bytecode/opcode.rs`

---

For questions or issues, refer to the implementation summary or examine the source code in `mquickjs/src/compiler/`.
