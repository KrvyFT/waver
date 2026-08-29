//! Process-shared engine status. GUI reads; the audio callback writes.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Snapshot of the running output stream. All fields are atomics so the GUI
/// never locks the audio thread.
#[derive(Debug)]
pub struct EngineStatus {
    sample_rate: AtomicU32,
    block: AtomicU32,
    channels: AtomicU32,
    running: AtomicBool,
    xruns: AtomicU32,
}

impl EngineStatus {
    /// Zeros; the engine fills real values when the stream opens.
    pub fn new() -> Self {
        Self {
            sample_rate: AtomicU32::new(0),
            block: AtomicU32::new(0),
            channels: AtomicU32::new(0),
            running: AtomicBool::new(false),
            xruns: AtomicU32::new(0),
        }
    }

    /// Output sample rate in Hz, or `0` if the stream never started.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    /// Internal processing block size in frames.
    pub fn block(&self) -> u32 {
        self.block.load(Ordering::Relaxed)
    }

    /// Interleaved channel count of the output stream.
    pub fn channels(&self) -> u32 {
        self.channels.load(Ordering::Relaxed)
    }

    /// True after `Stream::play` succeeds.
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Count of cpal stream error callbacks (underruns / device errors).
    pub fn xruns(&self) -> u32 {
        self.xruns.load(Ordering::Relaxed)
    }

    /// Engine-only: record negotiated stream format.
    pub fn set_format(&self, sample_rate: u32, block: u32, channels: u32) {
        self.sample_rate.store(sample_rate, Ordering::Relaxed);
        self.block.store(block, Ordering::Relaxed);
        self.channels.store(channels, Ordering::Relaxed);
    }

    /// Engine-only: mark the callback as live.
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Relaxed);
    }

    /// Engine-only: increment the xrun counter. Must not allocate or block.
    pub fn bump_xrun(&self) {
        self.xruns.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self::new()
    }
}
