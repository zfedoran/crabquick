// Quick test to verify the fix

use mquickjs::Engine;

fn main() {
    let mut engine = Engine::new(8192);

    println!("Testing: eval(\"2 + 2\")");
    match engine.eval("2 + 2") {
        Ok(result) => {
            if let Some(i) = result.to_int() {
                println!("✓ Result: {} (expected: 4)", i);
                if i == 4 {
                    println!("✓ SUCCESS: eval now returns expression value!");
                } else {
                    println!("✗ FAIL: Got {}, expected 4", i);
                }
            } else {
                println!("✗ FAIL: Result is not an integer");
            }
        }
        Err(err) => {
            println!("✗ ERROR: {:?}", err);
        }
    }

    println!("\nTesting: eval(\"10 * 5 + 3\")");
    match engine.eval("10 * 5 + 3") {
        Ok(result) => {
            if let Some(i) = result.to_int() {
                println!("✓ Result: {} (expected: 53)", i);
                if i == 53 {
                    println!("✓ SUCCESS!");
                } else {
                    println!("✗ FAIL: Got {}, expected 53", i);
                }
            } else {
                println!("✗ FAIL: Result is not an integer");
            }
        }
        Err(err) => {
            println!("✗ ERROR: {:?}", err);
        }
    }

    println!("\nTesting: eval(\"true\")");
    match engine.eval("true") {
        Ok(result) => {
            if let Some(b) = result.to_bool() {
                println!("✓ Result: {} (expected: true)", b);
                if b {
                    println!("✓ SUCCESS!");
                } else {
                    println!("✗ FAIL: Got false, expected true");
                }
            } else {
                println!("✗ FAIL: Result is not a boolean");
            }
        }
        Err(err) => {
            println!("✗ ERROR: {:?}", err);
        }
    }

    println!("\nTesting: eval(\"var x = 5;\") - should return undefined");
    match engine.eval("var x = 5;") {
        Ok(result) => {
            if result.is_undefined() {
                println!("✓ Result: undefined (correct for var declaration)");
                println!("✓ SUCCESS!");
            } else {
                println!("✗ FAIL: Expected undefined for var declaration");
            }
        }
        Err(err) => {
            println!("✗ ERROR: {:?}", err);
        }
    }

    println!("\nAll tests completed!");
}
