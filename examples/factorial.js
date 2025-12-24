// Factorial function (recursive)
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// Calculate factorials from 1 to 10
for (var i = 1; i <= 10; i = i + 1) {
    console.log(i + "! = " + factorial(i));
}
