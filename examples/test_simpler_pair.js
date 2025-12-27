function makeFuncs() {
  var value = 0;
  
  function get() {
    return value;
  }
  
  function set(v) {
    value = v;
  }
  
  get();
  set(42);
  console.log("After internal set, value is:");
  console.log(get());
}

makeFuncs();
