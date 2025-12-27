// Debug test
var arr = [10, 20, 30];
console.log("Testing for...of");

var count = 0;
for (var v of arr) {
    console.log("v = " + v);
    count = count + 1;
    if (count > 5) break; // Safety limit
}
console.log("Count: " + count);
