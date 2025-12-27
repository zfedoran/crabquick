function outer() {
  function inner() {
    return 42;
  }
  return inner();
}
console.log(outer())
