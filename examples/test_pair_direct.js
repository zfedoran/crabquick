function makePair() {
  var value = 0;
  
  function get() {
    console.log("in get, returning value");
    return value;
  }
  
  function set(v) {
    console.log("in set, setting value to");
    console.log(v);
    value = v;
    console.log("value is now");
    console.log(value);
  }
  
  return {
    get: get,
    set: set
  };
}

var pair = makePair();
console.log("Initial:");
console.log(pair.get());
console.log("Setting to 42:");
pair.set(42);
console.log("After set:");
console.log(pair.get());
