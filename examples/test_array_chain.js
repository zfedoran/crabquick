// Test chaining array callback methods
var nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// map -> filter -> reduce
var result = nums
    .map(function(x) { return x * 2; })
    .filter(function(x) { return x > 10; })
    .reduce(function(acc, x) { return acc + x; }, 0);
console.log("map->filter->reduce:", result);

// filter -> map
var filtered = nums
    .filter(function(x) { return x % 2 === 0; })
    .map(function(x) { return x * x; });
console.log("filter->map:", filtered[0], filtered[1], filtered[2], filtered[3], filtered[4]);

// Multiple filters
var multi = nums
    .filter(function(x) { return x > 2; })
    .filter(function(x) { return x < 8; })
    .filter(function(x) { return x % 2 === 0; });
console.log("Multi filter:", multi[0], multi[1]);

// map -> find
var found = nums
    .map(function(x) { return x * 3; })
    .find(function(x) { return x > 20; });
console.log("map->find:", found);
