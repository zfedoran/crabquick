function outer() {
  function inner() {
    console.log("inner start");
    return 123;
  }
  return inner();
}
outer()
