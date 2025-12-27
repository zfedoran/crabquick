function outer() {
  var x = 10;
  return function() {
    return x;
  };
}
var f = outer();
console.log(f());
