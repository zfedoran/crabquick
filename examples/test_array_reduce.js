// Test Array.prototype.reduce
var arr = [1, 2, 3, 4, 5];

// Sum with initial value
var sum = arr.reduce(function(acc, x) {
    return acc + x;
}, 0);
console.log("Sum (with initial):", sum);

// Sum without initial value
var sum2 = arr.reduce(function(acc, x) {
    return acc + x;
});
console.log("Sum (without initial):", sum2);

// Product
var product = arr.reduce(function(acc, x) {
    return acc * x;
}, 1);
console.log("Product:", product);

// Max value
var max = arr.reduce(function(acc, x) {
    return x > acc ? x : acc;
});
console.log("Max:", max);

// Count
var count = arr.reduce(function(acc, x) {
    return acc + 1;
}, 0);
console.log("Count:", count);
