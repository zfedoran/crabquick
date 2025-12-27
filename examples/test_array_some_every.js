// Test Array.prototype.some and every
var arr = [2, 4, 6, 8, 10];

// some - has even (true)
var hasEven = arr.some(function(x) {
    return x % 2 === 0;
});
console.log("Has even:", hasEven);

// some - has odd (false)
var hasOdd = arr.some(function(x) {
    return x % 2 !== 0;
});
console.log("Has odd:", hasOdd);

// some - has > 5 (true)
var hasBig = arr.some(function(x) {
    return x > 5;
});
console.log("Has > 5:", hasBig);

// every - all even (true)
var allEven = arr.every(function(x) {
    return x % 2 === 0;
});
console.log("All even:", allEven);

// every - all < 20 (true)
var allSmall = arr.every(function(x) {
    return x < 20;
});
console.log("All < 20:", allSmall);

// every - all > 5 (false)
var allBig = arr.every(function(x) {
    return x > 5;
});
console.log("All > 5:", allBig);

// Mixed array
var mixed = [1, 2, 3, 4, 5];
console.log("Mixed has even:", mixed.some(function(x) { return x % 2 === 0; }));
console.log("Mixed all even:", mixed.every(function(x) { return x % 2 === 0; }));
