use apodize::hanning_iter;
use num_complex::Complex;
use rustfft::FftPlanner;

const FFT_SIZE: usize = 1024;
const SPECTRUM_SIZE: usize = 64;
const NOISE_FLOOR: f32 = 0.001;
const MIN_THRESHOLD: f32 = 0.05;

pub fn compute_spectrum(audio: &[f32]) -> Vec<f32> {

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

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
                }
            }

            if count > 0 {
                spectrum[i] = (sum / count as f32).sqrt() * 0.25;
            }
        }
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

    } else {
        for val in &mut smoothed {
            *val = (*val * 5.0).min(0.05);
        }
    }

    smoothed
}
