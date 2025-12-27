function outer() {
  return function() {
    return 42;
  };
}
var f = outer();
f()
