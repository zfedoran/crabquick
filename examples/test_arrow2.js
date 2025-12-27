// Arrow with closures and chaining
var arr = [1, 2, 3, 4, 5];

// Filter and map with arrows
var result = arr.filter(x => x % 2 === 0).map(x => x * 10);
console.log("Even*10:", result[0], result[1]);

// Reduce with arrow
var sum = arr.reduce((a, b) => a + b, 0);
console.log("Sum:", sum);

// Nested arrows
var addCurried = a => b => c => a + b + c;
console.log("Curried 1+2+3:", addCurried(1)(2)(3));

// Arrow returning object (needs parentheses in real JS, but let's test block body)
var makeObj = x => { return { val: x }; };
var obj = makeObj(42);
console.log("obj.val:", obj.val);

// forEach with arrow
var count = 0;
[1, 2, 3].forEach(n => { count = count + n; });
console.log("forEach sum:", count);

// find with arrow
var found = [10, 20, 30, 40].find(x => x > 25);
console.log("Found:", found);
