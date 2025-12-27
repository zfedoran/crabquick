function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
var i = 1;
console.log(factorial(i));
i = 2;
console.log(factorial(i));
