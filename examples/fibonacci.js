// Fibonacci sequence generator
function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Print first 10 fibonacci numbers
for (var i = 0; i < 10; i = i + 1) {
    console.log(fibonacci(i));
}
