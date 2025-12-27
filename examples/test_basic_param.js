function test(x) {
  return function() {
    return x;
  };
}

var f1 = test(100);
console.log(f1());
var f2 = test(200);
console.log(f1());
console.log(f2());
