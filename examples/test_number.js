// Test global number functions
console.log("=== Testing Global Number Functions ===");

console.log("parseInt('42') = " + parseInt("42"));
console.log("parseInt('  123  ') = " + parseInt("  123  "));
console.log("parseInt('-50') = " + parseInt("-50"));
console.log("parseInt('0xff', 16) = " + parseInt("0xff", 16));
console.log("parseInt('ff', 16) = " + parseInt("ff", 16));
console.log("parseInt('abc') = " + parseInt("abc"));

console.log("");
console.log("parseFloat('3.14') = " + parseFloat("3.14"));
console.log("parseFloat('-2.5') = " + parseFloat("-2.5"));
console.log("parseFloat('123.456abc') = " + parseFloat("123.456abc"));
console.log("parseFloat('abc') = " + parseFloat("abc"));

console.log("");
console.log("isNaN(NaN) = " + isNaN(NaN));
console.log("isNaN(42) = " + isNaN(42));
console.log("isNaN('hello') = " + isNaN("hello"));

console.log("");
console.log("isFinite(42) = " + isFinite(42));
console.log("isFinite(Infinity) = " + isFinite(Infinity));
console.log("isFinite(NaN) = " + isFinite(NaN));

// Test Number static methods
console.log("");
console.log("=== Testing Number Static Methods ===");
console.log("Number.isNaN(NaN) = " + Number.isNaN(NaN));
console.log("Number.isNaN(42) = " + Number.isNaN(42));

console.log("Number.isFinite(42) = " + Number.isFinite(42));
console.log("Number.isFinite(Infinity) = " + Number.isFinite(Infinity));

console.log("Number.isInteger(42) = " + Number.isInteger(42));
console.log("Number.isInteger(3.14) = " + Number.isInteger(3.14));

// Test Number constants
console.log("");
console.log("=== Testing Number Constants ===");
console.log("Number.MAX_VALUE exists: " + (Number.MAX_VALUE > 0));
console.log("Number.MIN_VALUE exists: " + (Number.MIN_VALUE > 0));
console.log("isNaN(Number.NaN) = " + isNaN(Number.NaN));
console.log("Number.POSITIVE_INFINITY = " + Number.POSITIVE_INFINITY);
console.log("Number.NEGATIVE_INFINITY = " + Number.NEGATIVE_INFINITY);

console.log("");
console.log("=== All Number tests complete ===");
