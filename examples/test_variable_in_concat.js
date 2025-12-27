function factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
var i = 5;
console.log(i + "! = " + factorial(i));
