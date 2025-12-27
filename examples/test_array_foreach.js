// Test Array.prototype.forEach
var arr = [1, 2, 3, 4, 5];

// Basic forEach
console.log("forEach basic:");
arr.forEach(function(x) {
    console.log(x);
});

// forEach with index
console.log("forEach with index:");
arr.forEach(function(elem, idx) {
    console.log(idx + ": " + elem);
});
