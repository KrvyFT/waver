//! Temporary debug instrumentation for UI interaction bugs.
//! Writes NDJSON lines to /opt/cursor/logs/debug.log

use std::fs::OpenOptions;
use std::io::Write;

/// Append one NDJSON debug record. Failures are silently ignored.
pub fn agent_log(hypothesis_id: &str, location: &str, message: &str, data: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!(
        "{{\"hypothesisId\":\"{hypothesis_id}\",\"location\":\"{location}\",\"message\":\"{message}\",\"data\":{data},\"timestamp\":{ts}}}\n"
    );
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/opt/cursor/logs/debug.log")
    {
        let _ = f.write_all(line.as_bytes());
    }
}
