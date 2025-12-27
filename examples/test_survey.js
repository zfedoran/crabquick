// Test 1: typeof function
function foo() {}
console.log("typeof function:", typeof foo);

// Test 2: typeof closure  
var closure = (function() { return function() {}; })();
console.log("typeof closure:", typeof closure);

// Test 3: Array with functions
var arr = [function() { return 1; }];
console.log("array func call:", arr[0]());

// Test 4: Object with function property
var obj = { fn: function() { return 2; } };
console.log("obj.fn:", typeof obj.fn);
