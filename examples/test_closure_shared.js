// Test closures sharing same captured variable

function makeCounters() {
    var count = 0;

    var inc = function() {
        count = count + 1;
        return count;
    };

    var dec = function() {
        count = count - 1;
        return count;
    };

    var reset = function() {
        count = 0;
        return count;
    };

    return {
        inc: inc,
        dec: dec,
        reset: reset
    };
}

var counters = makeCounters();
console.log("inc:", counters.inc());
console.log("inc:", counters.inc());
console.log("inc:", counters.inc());
console.log("dec:", counters.dec());
console.log("dec:", counters.dec());
console.log("reset:", counters.reset());
console.log("inc:", counters.inc());
