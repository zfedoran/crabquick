function outer() {
  function inner() {
    return 42;
  }
  console.log("about to get inner");
  console.log(typeof inner);
  return 1;
}
console.log(outer())
