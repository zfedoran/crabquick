function outer() {
  function inner() {
    console.log("called inner");
    return 42;
  }
  inner();
}
outer()
