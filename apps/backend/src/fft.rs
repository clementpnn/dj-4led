use apodize::hanning_iter;
use num_complex::Complex;
use rustfft::FftPlanner;

const FFT_SIZE: usize = 1024;
const SPECTRUM_SIZE: usize = 64;
const NOISE_FLOOR: f32 = 0.0005; // Seuil de bruit pour filtrer le silence (divisé par 2)
const MIN_THRESHOLD: f32 = 0.025; // Seuil minimum pour normalisation (divisé par 2)

pub fn compute_spectrum(audio: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    // Vérifier d'abord le niveau d'entrée audio
    let audio_level: f32 = audio.iter().map(|&x| x.abs()).sum::<f32>() / audio.len() as f32;

    // Si le niveau est trop bas (silence), retourner un spectre vide
    if audio_level < NOISE_FLOOR {
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

    // Utiliser une distribution linéaire simple pour utiliser toutes les barres
    let useful_bins = FFT_SIZE / 4; // On utilise seulement le quart inférieur du spectre (jusqu'à 12kHz pour 48kHz)

    for i in 0..SPECTRUM_SIZE {
        // Distribution linéaire des bins
        let start = (i * useful_bins) / SPECTRUM_SIZE;
        let end = ((i + 1) * useful_bins) / SPECTRUM_SIZE;

        if start < end && end <= FFT_SIZE / 2 {
            // Calculer la magnitude moyenne pour cette bande
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
                spectrum[i] = (sum / count as f32).sqrt() * 0.5; // Diviser par 2 le volume détecté
            }
        }
    }

    // Appliquer une pondération perceptuelle
    for i in 0..SPECTRUM_SIZE {
        // Boost progressif des basses et médiums
        let freq_factor = if i < 8 {
            1.5 // Boost important des basses
        } else if i < 16 {
            1.3 // Boost modéré des bas-médiums
        } else if i < 32 {
            1.1 // Léger boost des médiums
        } else {
            0.9 // Légère atténuation des aigus
        };

        spectrum[i] *= freq_factor;
    }

    // Lissage spatial pour éviter les trous
    let mut smoothed = vec![0.0; SPECTRUM_SIZE];
    for i in 0..SPECTRUM_SIZE {
        let mut sum = spectrum[i] * 0.6; // Poids central
        let mut weight = 0.6;

        // Moyenne avec les voisins pour un effet plus continu
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
        // Normalisation avec préservation de la dynamique
        let norm_factor = 1.0 / max;
        let dynamic_factor = (avg / max).max(0.3); // Préserver la dynamique

        for val in &mut smoothed {
            // Normaliser avec courbe de réponse et diviser par 2
            *val = (*val * norm_factor * 0.5).powf(0.7 + dynamic_factor * 0.3);
            *val = val.min(1.0);
        }
    } else {
        // Si le signal est trop faible, appliquer un gain minimal
        for val in &mut smoothed {
            *val = (*val * 10.0).min(0.1); // Gain léger pour les signaux très faibles (divisé par 2)
        }
    }

    // Post-traitement pour garantir qu'on utilise toutes les barres
    // Si certaines barres sont à zéro entre des barres non-nulles, les interpoler
    for i in 1..SPECTRUM_SIZE - 1 {
        if smoothed[i] == 0.0 && smoothed[i - 1] > 0.0 && smoothed[i + 1] > 0.0 {
            smoothed[i] = (smoothed[i - 1] + smoothed[i + 1]) * 0.5;
        }
    }

    smoothed
}
