// Nested function - tests calling functions defined inside functions
function outer() {
  function inner() {
    console.log("inner called");
    return 42;
  }
  return inner();
}

console.log(outer()); // inner called, 42
