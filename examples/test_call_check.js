function outer() {
  function inner() {
    return 123;
  }
  return inner;
}
var f = outer();
var result = f();
console.log(result)
