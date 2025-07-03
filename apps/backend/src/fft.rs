use apodize::hanning_iter;
use num_complex::Complex;
use rustfft::FftPlanner;

const FFT_SIZE: usize = 1024;
const SPECTRUM_SIZE: usize = 64;

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

    // Convert to magnitude spectrum (logarithmic bins)
    let mut spectrum = vec![0.0; SPECTRUM_SIZE];

    for i in 0..SPECTRUM_SIZE {
        let start = ((i * FFT_SIZE / 2) / SPECTRUM_SIZE).min(FFT_SIZE / 2 - 1);
        let end = (((i + 1) * FFT_SIZE / 2) / SPECTRUM_SIZE).min(FFT_SIZE / 2);

        if start < end {
            let sum: f32 = input[start..end].iter().map(|c| c.norm()).sum();

            spectrum[i] = (sum / (end - start) as f32).sqrt();
        }
    }

    // Normalize
    let max = spectrum.iter().cloned().fold(0.0, f32::max);
    if max > 0.0 {
        for val in &mut spectrum {
            *val /= max;
        }
    }

    spectrum
}
