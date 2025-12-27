// Test Array.prototype.find and findIndex
var arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// find - first even number
var firstEven = arr.find(function(x) {
    return x % 2 === 0;
});
console.log("First even:", firstEven);

// find - first greater than 5
var firstBig = arr.find(function(x) {
    return x > 5;
});
console.log("First > 5:", firstBig);

// find - not found (returns undefined)
var notFound = arr.find(function(x) {
    return x > 100;
});
console.log("Not found:", notFound);

// findIndex - first even
var evenIdx = arr.findIndex(function(x) {
    return x % 2 === 0;
});
console.log("First even index:", evenIdx);

// findIndex - first > 5
var bigIdx = arr.findIndex(function(x) {
    return x > 5;
});
console.log("First > 5 index:", bigIdx);

// findIndex - not found (returns -1)
var notFoundIdx = arr.findIndex(function(x) {
    return x > 100;
});
console.log("Not found index:", notFoundIdx);
