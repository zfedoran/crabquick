function outer() {
  function inner() {
    return 123;
  }
  return inner;
}
var f = outer();
console.log(typeof f);
console.log(f);
