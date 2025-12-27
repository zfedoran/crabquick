// Test Function.prototype.call/apply/bind

// Simple function to test with
function greet(greeting) {
    return greeting + " " + this.name;
}

// Test object
var person = { name: "World" };

// Test call
var result1 = greet.call(person, "Hello");
console.log("call result:", result1);

// Test apply
var result2 = greet.apply(person, ["Hi"]);
console.log("apply result:", result2);

// Test bind (simplified - returns same function)
var bound = greet.bind(person);
console.log("bind returns function:", typeof bound === "function");

// Test with Math functions
console.log("max via apply:", Math.max.apply(null, [1, 2, 3, 4, 5]));

// Test with built-in methods
var arr = [1, 2, 3];
console.log("push via call:", Array.prototype.push.call(arr, 4));
console.log("arr length after push:", arr.length);
