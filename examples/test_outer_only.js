function outer() {
  function inner() {
    return 42;
  }
  console.log("in outer");
  return 1;
}
console.log(outer())
