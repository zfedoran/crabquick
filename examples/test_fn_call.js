function outer() {
  return function() {
    return 42;
  };
}
outer()()
