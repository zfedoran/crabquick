// Test Array.isArray
console.log("Array.isArray([1,2,3]):", Array.isArray([1, 2, 3]));
console.log("Array.isArray([]):", Array.isArray([]));
console.log("Array.isArray({}):", Array.isArray({}));
console.log("Array.isArray(null):", Array.isArray(null));
console.log("Array.isArray(undefined):", Array.isArray(undefined));
console.log("Array.isArray(123):", Array.isArray(123));
console.log("Array.isArray('array'):", Array.isArray("array"));

// Test with result from map/filter
var arr = [1, 2, 3];
var mapped = arr.map(function(x) { return x * 2; });
var filtered = arr.filter(function(x) { return x > 1; });
console.log("Array.isArray(mapped):", Array.isArray(mapped));
console.log("Array.isArray(filtered):", Array.isArray(filtered));
