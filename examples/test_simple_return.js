function outer() {
  return function() {
    return 123;
  };
}
var fn = outer();
typeof fn
