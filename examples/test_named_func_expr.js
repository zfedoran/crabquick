// Test named function expressions (for recursion)

// Factorial using named function expression
var factorial = function fact(n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
};
console.log("5! =", factorial(5));
console.log("10! =", factorial(10));

// Fibonacci using named function expression
var fib = function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
};
console.log("fib(10) =", fib(10));

// Named function expression in object
var math = {
    factorial: function fact(n) {
        return n <= 1 ? 1 : n * fact(n - 1);
    }
};
console.log("math.factorial(6) =", math.factorial(6));
