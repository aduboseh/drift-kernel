#![no_main]

use libfuzzer_sys::fuzz_target;
use drift_kernel::ScgAccumulator;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    
    // First 8 bytes = initial value
    let initial_bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let initial = f64::from_le_bytes(initial_bytes);
    
    if !initial.is_finite() {
        return;
    }
    
    let mut acc = ScgAccumulator::new(initial);
    
    // Remaining bytes = values to add
    for chunk in data[8..].chunks_exact(8) {
        let bytes: [u8; 8] = chunk.try_into().unwrap();
        let value = f64::from_le_bytes(bytes);
        
        if value.is_finite() {
            acc.add(value);
        }
    }
    
    let total = acc.total();
    let raw = acc.raw_sum();
    let comp = acc.compensation();
    let drift = acc.drift();
    let ops = acc.ops();
    
    // Sanity checks
    assert!(total.is_finite() || !initial.is_finite(), "Non-finite total from finite inputs");
    assert!(raw.is_finite() || !initial.is_finite(), "Non-finite raw_sum from finite inputs");
    assert!(comp.is_finite(), "Non-finite compensation");
    assert!(drift.is_finite() || !initial.is_finite(), "Non-finite drift from finite inputs");
    
    // Test reset
    acc.reset();
    assert_eq!(acc.ops(), 0);
    assert_eq!(acc.total(), initial);
    
    // Verify ops count matches actual operations
    let expected_ops = data[8..].chunks_exact(8)
        .filter(|chunk| {
            let bytes: [u8; 8] = (*chunk).try_into().unwrap();
            f64::from_le_bytes(bytes).is_finite()
        })
        .count() as u64;
    assert_eq!(ops, expected_ops);
});
