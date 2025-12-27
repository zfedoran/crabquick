function outer() {
  var x = 10;
  return function() {
    x = 20;
    return x;
  };
}
var f = outer();
console.log(f());
