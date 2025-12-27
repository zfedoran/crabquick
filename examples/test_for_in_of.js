// Test for...in loop with arrays
console.log("=== for...in with array ===");
var arr = [10, 20, 30];
var keys = "";
for (var k in arr) {
    keys = keys + k + " ";
}
console.log("Keys: " + keys);

// Test for...of loop with arrays
console.log("\n=== for...of with array ===");
var arr2 = [1, 2, 3, 4, 5];
var sum = 0;
for (var v of arr2) {
    sum = sum + v;
}
console.log("Sum: " + sum);

// Test for...of with strings
console.log("\n=== for...of with string ===");
var str = "hello";
var chars = "";
for (var c of str) {
    chars = chars + c + "-";
}
console.log("Chars: " + chars);

// Test for...in with string
console.log("\n=== for...in with string ===");
var str2 = "abc";
var indices = "";
for (var i in str2) {
    indices = indices + i + " ";
}
console.log("Indices: " + indices);

// Test nested for...of
console.log("\n=== nested for...of ===");
var outer = [[1, 2], [3, 4]];
var total = 0;
for (var inner of outer) {
    for (var n of inner) {
        total = total + n;
    }
}
console.log("Total: " + total);

// Test for...of with break
console.log("\n=== for...of with break ===");
var arr3 = [1, 2, 3, 4, 5];
var partial = 0;
for (var x of arr3) {
    if (x > 3) break;
    partial = partial + x;
}
console.log("Partial (1+2+3): " + partial);

// Test for...of with continue
console.log("\n=== for...of with continue ===");
var arr4 = [1, 2, 3, 4, 5];
var oddSum = 0;
for (var y of arr4) {
    if (y % 2 === 0) continue;
    oddSum = oddSum + y;
}
console.log("Odd sum (1+3+5): " + oddSum);

console.log("\n=== All tests completed ===");
