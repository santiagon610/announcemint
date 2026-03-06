//! Progress reporting for generate flow: trait used by polly and preset to report steps.

/// Implemented in main; used by polly and preset to report progress steps without depending on Tauri.
pub trait ProgressReporter: Send + Sync {
    fn report(&self, step: &str);
}
