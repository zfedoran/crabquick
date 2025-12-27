// Test array literals in various contexts
console.log("Direct call:", [1,2,3].length);
console.log("Nested:", [[1,2],[3,4]][0][1]);

function f(arr) { return arr.map(function(x) { return x*2; }); }
var result = f([5, 10, 15]);
console.log("After map:", result[0], result[1], result[2]);

// Array in object
var obj = { arr: [100, 200] };
console.log("Obj arr:", obj.arr[0], obj.arr[1]);

// Multiple array args
function g(a, b) { return a[0] + b[0]; }
console.log("Multi args:", g([10], [20]));
