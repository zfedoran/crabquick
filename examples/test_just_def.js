function outer() {
  console.log("A");
  function inner() { return 1; }
  console.log("B");
  return 1;
}
console.log("start");
outer();
console.log("end")
