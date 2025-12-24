// Closure example - counter
function makeCounter() {
    var count = 0;
    return function() {
        count = count + 1;
        return count;
    };
}

var counter1 = makeCounter();
var counter2 = makeCounter();

console.log(counter1()); // 1
console.log(counter1()); // 2
console.log(counter1()); // 3

console.log(counter2()); // 1
console.log(counter2()); // 2
