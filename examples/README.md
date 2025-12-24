# MicroQuickJS Example Programs

This directory contains example JavaScript programs that demonstrate the capabilities of the MicroQuickJS engine.

## Running Examples

```bash
# Using cargo
cargo run --package mquickjs --bin mquickjs examples/hello.js

# Using the built binary
./target/debug/mquickjs examples/hello.js

# With memory statistics
cargo run --package mquickjs --bin mquickjs -- -m examples/fibonacci.js
```

## Examples

### hello.js
Basic "Hello, World!" program using console.log.

```bash
cargo run --package mquickjs --bin mquickjs examples/hello.js
```

**Expected Output:**
```
Hello, World!
```

### fibonacci.js
Demonstrates recursive function calls by computing the first 10 Fibonacci numbers.

```bash
cargo run --package mquickjs --bin mquickjs examples/fibonacci.js
```

**Expected Output:**
```
0
1
1
2
3
5
8
13
21
34
```

### fizzbuzz.js
Classic FizzBuzz implementation (1-20) demonstrating conditionals and modulo operator.

```bash
cargo run --package mquickjs --bin mquickjs examples/fizzbuzz.js
```

**Expected Output:**
```
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
17
Fizz
19
Buzz
```

### factorial.js
Calculates factorials from 1 to 10 using recursion.

```bash
cargo run --package mquickjs --bin mquickjs examples/factorial.js
```

**Expected Output:**
```
1! = 1
2! = 2
3! = 6
4! = 24
5! = 120
6! = 720
7! = 5040
8! = 40320
9! = 362880
10! = 3628800
```

### counter.js
Demonstrates closures by creating independent counter functions.

```bash
cargo run --package mquickjs --bin mquickjs examples/counter.js
```

**Expected Output:**
```
1
2
3
1
2
```

### arrays.js
Shows array operations: push, pop, join, and property access.

```bash
cargo run --package mquickjs --bin mquickjs examples/arrays.js
```

**Expected Output:**
```
Array length: 5
First element: 1
Last element: 5
After push: length = 6
Popped: 6
After pop: length = 5
Joined: 1, 2, 3, 4, 5
```

### objects.js
Demonstrates object creation, property access, and methods.

```bash
cargo run --package mquickjs --bin mquickjs examples/objects.js
```

**Expected Output:**
```
Name: Alice
Age: 30
Country: USA
New age: 31
2 + 3 = 5
4 * 5 = 20
```

### math.js
Shows the Math object with constants and methods.

```bash
cargo run --package mquickjs --bin mquickjs examples/math.js
```

**Expected Output:**
```
PI: 3.141592653589793
E: 2.718281828459045
abs(-5): 5
floor(3.7): 3
ceil(3.2): 4
round(3.5): 4
min(5, 3, 8, 1): 1
max(5, 3, 8, 1): 8
pow(2, 8): 256
sqrt(16): 4
```

## Note

These examples will work once the compiler and VM are fully integrated. Currently, they serve as:

1. **Test cases** for verifying engine functionality
2. **Documentation** showing supported JavaScript features
3. **Benchmarks** for performance testing
4. **Demonstrations** of engine capabilities

## Creating Your Own Examples

Create a new `.js` file in this directory with your JavaScript code:

```javascript
// myexample.js
function greet(name) {
    console.log("Hello, " + name + "!");
}

greet("World");
```

Then run it:

```bash
cargo run --package mquickjs --bin mquickjs examples/myexample.js
```

## Supported Features

The examples demonstrate these JavaScript features:

- ✓ Variable declarations (`var`)
- ✓ Function declarations
- ✓ Function calls
- ✓ Recursion
- ✓ Closures
- ✓ Object literals
- ✓ Array literals
- ✓ Property access (dot notation)
- ✓ Array indexing
- ✓ Arithmetic operators
- ✓ Comparison operators
- ✓ Logical operators
- ✓ If/else statements
- ✓ For loops
- ✓ While loops
- ✓ String concatenation
- ✓ Built-in methods (push, pop, etc.)
- ✓ Math object
- ✓ Console.log

## Limitations

Current limitations in the examples:

- No ES6+ features (const, let, arrow functions, etc.)
- No destructuring
- No spread operator
- No template literals
- No async/await
- No modules (import/export)

These are intentional limitations to keep the engine minimal and focused on core JavaScript functionality.
