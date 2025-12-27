function outer() {
  console.log("before inner def");
  function inner() {
    return 99;
  }
  console.log("after inner def");
  console.log(typeof inner);
  return inner;
}
console.log("calling outer");
outer()
