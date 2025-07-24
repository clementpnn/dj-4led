use apodize::hanning_iter;
use num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const FFT_SIZE: usize = 1024;
const SPECTRUM_SIZE: usize = 64;
const NOISE_FLOOR: f32 = 0.001;
const MIN_THRESHOLD: f32 = 0.05;

static DEBUG_COUNTER: AtomicU32 = AtomicU32::new(0);
static LAST_LOG_TIME: Mutex<Option<Instant>> = Mutex::new(None);

pub fn compute_spectrum(audio: &[f32]) -> Vec<f32> {
    let counter = DEBUG_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Log toutes les 120 frames (environ toutes les 2 secondes à 60 FPS)
    let should_log = counter % 120 == 0 || counter <= 2;

    if should_log {
        let now = Instant::now();
        let mut last_time_guard = LAST_LOG_TIME.lock().unwrap();

        if let Some(last_time) = *last_time_guard {
            let fps = 120.0 / last_time.elapsed().as_secs_f32();
            println!("🎵 [FFT] Frame #{} (FPS: {:.1})", counter, fps);
        } else {
            println!("🎵 [FFT] Frame #{} (init)", counter);
        }
        *last_time_guard = Some(now);
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    // Vérifier le niveau d'entrée audio
    let audio_level: f32 = audio.iter().map(|&x| x.abs()).sum::<f32>() / audio.len() as f32;
    let audio_peak = audio.iter().map(|&x| x.abs()).fold(0.0, f32::max);

    if should_log {
        println!("  📊 [FFT] Input: {} samples, Level: {:.6}, Peak: {:.6}",
                 audio.len(), audio_level, audio_peak);
    }

    // Si le niveau est trop bas (silence), retourner un spectre vide
    if audio_level < NOISE_FLOOR {
        if should_log {
            println!("  🔇 [FFT] Silence détecté");
        }
        return vec![0.0; SPECTRUM_SIZE];
    }

    // Prepare input with windowing
    let mut input: Vec<Complex<f32>> = audio
        .iter()
        .take(FFT_SIZE)
        .zip(hanning_iter(FFT_SIZE))
        .map(|(&sample, window)| Complex::new(sample * window as f32, 0.0))
        .collect();

    // Pad if necessary
    input.resize(FFT_SIZE, Complex::new(0.0, 0.0));

    // Compute FFT
    fft.process(&mut input);

    // Convert to magnitude spectrum with linear distribution
    let mut spectrum = vec![0.0; SPECTRUM_SIZE];
    let useful_bins = FFT_SIZE / 4;

    let mut total_magnitude = 0.0;
    let mut active_bins = 0;

    for i in 0..SPECTRUM_SIZE {
        let start = (i * useful_bins) / SPECTRUM_SIZE;
        let end = ((i + 1) * useful_bins) / SPECTRUM_SIZE;

        if start < end && end <= FFT_SIZE / 2 {
            let mut sum = 0.0;
            let mut count = 0;

            for j in start..end {
                let magnitude = input[j].norm();
                if magnitude > NOISE_FLOOR {
                    sum += magnitude;
                    count += 1;
                    total_magnitude += magnitude;
                    active_bins += 1;
                }
            }

            if count > 0 {
                spectrum[i] = (sum / count as f32).sqrt() * 0.25;
            }
        }
    }

    if should_log {
        let spectrum_peak = spectrum.iter().cloned().fold(0.0, f32::max);
        let spectrum_avg = spectrum.iter().sum::<f32>() / SPECTRUM_SIZE as f32;
        println!("  📈 [FFT-SPECTRUM] Bins actifs: {}/{} | Magnitude totale: {:.6}",
                 active_bins, useful_bins, total_magnitude);
        println!("  📈 [FFT-SPECTRUM] Raw - Peak: {:.6} | Avg: {:.6}",
                 spectrum_peak, spectrum_avg);
    }

    // Appliquer une pondération perceptuelle
    for i in 0..SPECTRUM_SIZE {
        let freq_factor = if i < 8 {
            1.5 // Boost basses
        } else if i < 16 {
            1.3 // Boost bas-médiums
        } else if i < 32 {
            1.1 // Léger boost médiums
        } else {
            0.9 // Légère atténuation aigus
        };
        spectrum[i] *= freq_factor;
    }

    // Lissage spatial
    let mut smoothed = vec![0.0; SPECTRUM_SIZE];
    for i in 0..SPECTRUM_SIZE {
        let mut sum = spectrum[i] * 0.6;
        let mut weight = 0.6;

        for offset in 1..=2 {
            if i >= offset {
                let neighbor_weight = 0.2 / offset as f32;
                sum += spectrum[i - offset] * neighbor_weight;
                weight += neighbor_weight;
            }
            if i + offset < SPECTRUM_SIZE {
                let neighbor_weight = 0.2 / offset as f32;
                sum += spectrum[i + offset] * neighbor_weight;
                weight += neighbor_weight;
            }
        }
        smoothed[i] = sum / weight;
    }

    // Normalisation adaptative
    let max = smoothed.iter().cloned().fold(0.0, f32::max);
    let avg = smoothed.iter().sum::<f32>() / SPECTRUM_SIZE as f32;

    if max > MIN_THRESHOLD {
        let norm_factor = 1.0 / max;
        let dynamic_factor = (avg / max).max(0.3);

        for val in &mut smoothed {
            *val = (*val * norm_factor * 0.25).powf(0.7 + dynamic_factor * 0.3);
            *val = val.min(1.0);
        }

        if should_log {
            let final_max = smoothed.iter().cloned().fold(0.0, f32::max);
            println!("  🎛️ [FFT] Normalisé - Max: {:.6}", final_max);
        }
    } else {
        for val in &mut smoothed {
            *val = (*val * 5.0).min(0.05);
        }

        if should_log {
            println!("  🎛️ [FFT] Signal faible - amplifié");
        }
    }

    // Post-traitement pour combler les trous
    let mut holes_filled = 0;
    for i in 1..SPECTRUM_SIZE - 1 {
        if smoothed[i] == 0.0 && smoothed[i - 1] > 0.0 && smoothed[i + 1] > 0.0 {
            smoothed[i] = (smoothed[i - 1] + smoothed[i + 1]) * 0.5;
            holes_filled += 1;
        }
    }

    if should_log {
        let active_bands = smoothed.iter().filter(|&&x| x > 0.01).count();
        let energy_total = smoothed.iter().sum::<f32>();

        println!("  ✅ [FFT] Bandes actives: {}/{}, Énergie: {:.6}, Trous: {}",
                 active_bands, SPECTRUM_SIZE, energy_total, holes_filled);

        let bass = if smoothed.len() >= 8 { smoothed[0..8].iter().sum::<f32>() / 8.0 } else { 0.0 };
        let mid = if smoothed.len() >= 32 { smoothed[8..32].iter().sum::<f32>() / 24.0 } else { 0.0 };
        let treble = if smoothed.len() >= 64 { smoothed[32..64].iter().sum::<f32>() / 32.0 } else { 0.0 };
        println!("  🎼 [FFT] Bass: {:.3} | Mid: {:.3} | Treble: {:.3}", bass, mid, treble);

        let level_bars = (energy_total * 20.0) as usize;
        let visual = "█".repeat(level_bars.min(20));
        println!("  🔊 [FFT] {:<20}", visual);
    }

    smoothed
}
