// audio/mod.rs

pub mod capture;
pub mod fft;

// Re-exports pour faciliter l'utilisation
pub use capture::AudioCapture;
pub use fft::{compute_spectrum, analyze_signal_quality, detect_music_signal};
