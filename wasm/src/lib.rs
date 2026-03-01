// LEARNING NOTE: No `fn main()` here!
// A Wasm module is a LIBRARY, not an executable.
// The browser is the runtime. It loads and calls into this module.
// Compare this to src/main.rs - that is a native binary with a main().
// This is lib.rs - it exposes functions for others to call.

use wasm_bindgen::prelude::*;

// ═══════════════════════════════════════════════════════════════════
// CONCEPT 1: EXPORTS
//
// #[wasm_bindgen] marks this function as a Wasm EXPORT.
// In the compiled .wasm binary, the "export" section contains:
//   (export "fibonacci" (func $fibonacci))
//
// This is what allows JavaScript to call: wasmModule.fibonacci(n)
//
// Wasm type signature: (i32) -> i64
// JavaScript sees it as: (Number) -> BigInt
// (because i64 can exceed Number.MAX_SAFE_INTEGER, wasm-bindgen uses BigInt)
// ═══════════════════════════════════════════════════════════════════
#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u64 {
    fib_inner(n)
}

// LEARNING NOTE: This function is NOT marked #[wasm_bindgen].
// Therefore it does NOT appear in the Wasm export section.
// It exists in the "code" section of the binary, but JavaScript
// cannot call it - it's an internal implementation detail.
// This demonstrates that Wasm respects information hiding.
fn fib_inner(n: u32) -> u64 {
    // Intentionally naive recursive implementation.
    // At n=42 this does ~866 million recursive calls.
    // JS handles this slowly (dynamic typing overhead),
    // but Wasm executes integer arithmetic on a typed stack machine -
    // no boxing, no GC, no type checks. That's where the speedup comes from.
    match n {
        0 => 0,
        1 => 1,
        _ => fib_inner(n - 1) + fib_inner(n - 2),
    }
}

// ═══════════════════════════════════════════════════════════════════
// CONCEPT 2: LINEAR MEMORY (zero-copy sharing)
//
// Wasm linear memory is a flat, byte-addressable array shared between
// the Wasm module and JavaScript. Both can read and write it.
//
// When JS passes a Uint32Array to this function, wasm-bindgen arranges
// for Rust to receive a slice pointing into the SAME memory region.
// No data is copied. This is the zero-copy model.
//
// After calling this function, JS can immediately read the results
// from the original Uint32Array - Rust wrote directly into it.
// ═══════════════════════════════════════════════════════════════════
#[wasm_bindgen]
pub fn fill_fibonacci_sequence(buffer: &mut [u32]) {
    if buffer.is_empty() {
        return;
    }
    if buffer.len() == 1 {
        buffer[0] = 0;
        return;
    }
    buffer[0] = 0;
    buffer[1] = 1;
    for i in 2..buffer.len() {
        // saturating_add: clamp at u32::MAX instead of wrapping/panicking
        buffer[i] = buffer[i - 1].saturating_add(buffer[i - 2]);
    }
}

// ═══════════════════════════════════════════════════════════════════
// CONCEPT 3: IMPORTS (Wasm calling INTO JavaScript)
//
// Wasm is sandboxed - it cannot do I/O on its own.
// It must IMPORT capabilities from the host (JavaScript).
//
// extern "C" { fn log(s: &str); } tells wasm-bindgen:
//   "console.log exists in the JS host; let me call it."
//
// In the compiled .wasm binary, the "import" section contains:
//   (import "wbg" "__wbg_log_..." (func ...))
//
// JS is the "host". Wasm is the "guest". The guest asks the host
// for capabilities it cannot provide itself.
// ═══════════════════════════════════════════════════════════════════
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// ═══════════════════════════════════════════════════════════════════
// CONCEPT 4: RETURNING COMPLEX TYPES (strings cross the boundary)
//
// Wasm functions natively return only numeric primitives: i32, i64, f32, f64.
// To return a String, wasm-bindgen:
//   1. Allocates bytes for the string in Wasm linear memory
//   2. Returns a (pointer: i32, length: i32) pair
//   3. The JS glue reads those bytes and reconstructs a JS String
//
// This is the key insight: "complex types" are just linear memory
// + a convention about how to interpret the bytes.
// ═══════════════════════════════════════════════════════════════════
#[wasm_bindgen]
pub fn get_module_info() -> String {
    // Calling log() here demonstrates the import concept:
    // This Rust code, compiled to Wasm, calls back into JavaScript.
    log("[Wasm] get_module_info() called - this log came from Rust via Wasm import!");
    String::from(
        "fib_wasm v0.1.0 | \
         Exports: fibonacci, fill_fibonacci_sequence, get_module_info | \
         Hint: read www/pkg/fib_wasm.js to see how types cross the boundary"
    )
}
