// Closure adder - tests parameter capture in closures
function makeAdder(x) {
  return function(y) {
    return x + y;
  };
}

var add5 = makeAdder(5);
var add10 = makeAdder(10);

console.log(add5(3));  // 8
console.log(add10(3)); // 13
console.log(add5(10)); // 15
