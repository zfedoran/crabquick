// Test Array.prototype.filter
var arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Filter even numbers
var evens = arr.filter(function(x) {
    return x % 2 === 0;
});
console.log("Evens:", evens[0], evens[1], evens[2], evens[3], evens[4]);
console.log("Evens length:", evens.length);

// Filter greater than 5
var big = arr.filter(function(x) {
    return x > 5;
});
console.log("Greater than 5:", big[0], big[1], big[2], big[3], big[4]);

// Chained filter
var result = arr.filter(function(x) { return x > 3; }).filter(function(x) { return x < 8; });
console.log("Between 3 and 8:", result[0], result[1], result[2], result[3]);
