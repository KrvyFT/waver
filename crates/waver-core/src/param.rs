//! Lock-free `f32` cell for GUI → audio parameter writes.

use std::sync::atomic::{AtomicU32, Ordering};

/// Audio-rate-safe parameter. GUI stores with `Relaxed`; the callback loads per block.
///
/// Bit pattern is `f32::to_bits` / `from_bits`, so NaN payloads survive a roundtrip.
pub struct ParamCell {
    bits: AtomicU32,
}

impl ParamCell {
    /// Initialise to `value`.
    pub fn new(value: f32) -> Self {
        Self {
            bits: AtomicU32::new(value.to_bits()),
        }
    }

    /// Store a new value. Safe from the GUI thread.
    pub fn set(&self, value: f32) {
        self.bits.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Load the current value. Safe from the audio thread (no allocation).
    pub fn value(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::ParamCell;

    #[test]
    fn param_cell_roundtrip() {
        let cell = ParamCell::new(0.5);
        assert!((cell.value() - 0.5).abs() < f32::EPSILON);
        cell.set(-0.25);
        assert!((cell.value() + 0.25).abs() < f32::EPSILON);
    }
}
