function makePair() {
  var value = 0;
  function get() {
    return value;
  }
  function set(v) {
    value = v;
  }
  console.log("get is:");
  console.log(typeof get);
  console.log("set is:");
  console.log(typeof set);
  return [get, set];
}

var pair = makePair();
console.log("pair[0] is:");
console.log(typeof pair[0]);
console.log("pair[1] is:");
console.log(typeof pair[1]);
