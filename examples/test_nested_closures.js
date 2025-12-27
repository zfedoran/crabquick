// Test nested closures

// Basic nested closure
function outer(x) {
    return function middle(y) {
        return function inner(z) {
            return x + y + z;
        };
    };
}
console.log("outer(1)(2)(3) =", outer(1)(2)(3));
console.log("outer(10)(20)(30) =", outer(10)(20)(30));

// Counter with nested closures
function makeCounter() {
    var count = 0;
    return {
        inc: function() { count = count + 1; return count; },
        dec: function() { count = count - 1; return count; },
        get: function() { return count; }
    };
}

var counter = makeCounter();
console.log("inc:", counter.inc());
console.log("inc:", counter.inc());
console.log("inc:", counter.inc());
console.log("dec:", counter.dec());
console.log("get:", counter.get());

// Independent counters
var c1 = makeCounter();
var c2 = makeCounter();
c1.inc();
c1.inc();
c2.inc();
console.log("c1:", c1.get());
console.log("c2:", c2.get());

// Currying
function curry(a) {
    return function(b) {
        return function(c) {
            return a * b + c;
        };
    };
}
console.log("curry(2)(3)(4) =", curry(2)(3)(4));
