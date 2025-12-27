function makePair() {
  var value = 0;
  function get() {
    return value;
  }
  function set(v) {
    value = v;
  }
  return [get, set];
}

var pair = makePair();
var get = pair[0];
var set = pair[1];

console.log(get());
set(42);
console.log(get());
set(100);
console.log(get());
