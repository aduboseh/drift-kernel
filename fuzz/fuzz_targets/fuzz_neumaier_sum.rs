#![no_main]

use libfuzzer_sys::fuzz_target;
use drift_kernel::neumaier_sum_slice;

fuzz_target!(|data: &[u8]| {
    // Interpret bytes as f64 values
    if data.len() < 8 {
        return;
    }
    
    let values: Vec<f64> = data
        .chunks_exact(8)
        .map(|chunk| {
            let bytes: [u8; 8] = chunk.try_into().unwrap();
            f64::from_le_bytes(bytes)
        })
        .filter(|v| v.is_finite()) // Skip NaN/Inf to test normal operation
        .collect();
    
    if values.is_empty() {
        return;
    }
    
    let result = neumaier_sum_slice(&values);
    
    // Basic sanity: result should be finite if all inputs are finite
    assert!(result.is_finite(), "Non-finite result from finite inputs");
});
