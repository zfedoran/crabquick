function outer() {
  return function() {
    return 42;
  };
}
var f = outer();
console.log(typeof f)
