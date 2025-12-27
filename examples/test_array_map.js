// Test Array.prototype.map
var arr = [1, 2, 3, 4, 5];

// Basic map - double each element
var doubled = arr.map(function(x) {
    return x * 2;
});
console.log("Original:", arr[0], arr[1], arr[2], arr[3], arr[4]);
console.log("Doubled:", doubled[0], doubled[1], doubled[2], doubled[3], doubled[4]);

// Map with index
var indexed = arr.map(function(x, i) {
    return x + i;
});
console.log("With index:", indexed[0], indexed[1], indexed[2], indexed[3], indexed[4]);

// Chained map
var result = arr.map(function(x) { return x * 2; }).map(function(x) { return x + 1; });
console.log("Chained:", result[0], result[1], result[2], result[3], result[4]);
