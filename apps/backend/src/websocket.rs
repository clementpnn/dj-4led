use crate::AppState;
use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum WsMessage {
    Effect {
        id: usize,
    },
    Param {
        name: String,
        value: String,
    },
    Frame {
        data: Vec<u8>,
    },
    Spectrum {
        data: Vec<f32>,
    },
    CompressedFrame {
        data: Vec<u8>,
        width: u16,
        height: u16,
    }, // Nouveau type pour frames compressées
}

pub struct WebSocketServer {
    state: Arc<AppState>,
}

impl WebSocketServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:8080").await?;
        listener.set_ttl(60)?; // Time to live optimisé

        println!("🌐 WebSocket server on ws://localhost:8080");

        while let Ok((stream, addr)) = listener.accept().await {
            println!("📱 New WebSocket connection from {}", addr);

            // Optimisations TCP
            stream.set_nodelay(true)?; // Désactiver Nagle pour latence minimale

            let state = self.state.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, state).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }

        Ok(())
    }
}

// Structure pour gérer le buffering et la compression
struct FrameProcessor {
    preview_buffer: Vec<u8>,
    compression_buffer: Vec<u8>,
    last_frame_hash: u64,
    last_spectrum_hash: u64,
    use_compression: bool,
}

impl FrameProcessor {
    fn new() -> Self {
        Self {
            preview_buffer: Vec::with_capacity(64 * 64 * 3),
            compression_buffer: Vec::with_capacity(32 * 1024),
            last_frame_hash: 0,
            last_spectrum_hash: 0,
            use_compression: true,
        }
    }

    // Hash rapide pour détecter les changements
    fn fast_hash(data: &[u8]) -> u64 {
        data.chunks(8).enumerate().fold(0u64, |acc, (i, chunk)| {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            acc.wrapping_add(u64::from_le_bytes(bytes).wrapping_mul(i as u64 + 1))
        })
    }

    // Downscale optimisé avec SIMD-friendly code
    fn downscale_frame(
        &mut self,
        src: &[u8],
        src_width: usize,
        dst_width: usize,
        dst_height: usize,
    ) {
        self.preview_buffer.clear();
        let scale = src_width / dst_width;

        for y in 0..dst_height {
            for x in 0..dst_width {
                let src_idx = ((y * scale) * src_width + (x * scale)) * 3;
                if src_idx + 2 < src.len() {
                    self.preview_buffer
                        .extend_from_slice(&src[src_idx..src_idx + 3]);
                } else {
                    self.preview_buffer.extend_from_slice(&[0, 0, 0]);
                }
            }
        }
    }

    // Compression des frames si activée
    fn compress_frame(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        if !self.use_compression || data.len() < 1024 {
            return None;
        }

        self.compression_buffer.clear();
        let mut encoder = GzEncoder::new(&mut self.compression_buffer, Compression::fast());

        if encoder.write_all(data).is_ok() && encoder.finish().is_ok() {
            // Utiliser la compression seulement si elle réduit la taille
            if self.compression_buffer.len() < data.len() * 3 / 4 {
                return Some(self.compression_buffer.clone());
            }
        }

        None
    }
}

async fn handle_connection(stream: TcpStream, state: Arc<AppState>) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Canal avec buffer limité pour éviter l'accumulation
    let (tx, mut rx) = mpsc::channel::<Message>(8);

    // Thread pour envoyer les frames
    let preview_state = state.clone();
    let preview_tx = tx.clone();

    tokio::spawn(async move {
        let mut processor = FrameProcessor::new();
        let mut interval = tokio::time::interval(Duration::from_millis(25)); // 40 FPS
        let mut last_sent = Instant::now();
        let mut frame_counter = 0u32;

        loop {
            interval.tick().await;
            frame_counter += 1;

            // Obtenir les données actuelles
            let frame = preview_state.led_frame.lock().clone();
            let spectrum = preview_state.spectrum.lock().clone();

            // Vérifier si la frame a changé (simple hash)
            let frame_hash = FrameProcessor::fast_hash(&frame);
            let spectrum_hash = FrameProcessor::fast_hash(unsafe {
                std::slice::from_raw_parts(spectrum.as_ptr() as *const u8, spectrum.len() * 4)
            });

            // Envoyer la frame seulement si elle a changé ou toutes les 30 frames (1Hz minimum)
            if frame_hash != processor.last_frame_hash || frame_counter % 30 == 0 {
                processor.last_frame_hash = frame_hash;

                // Downscale optimisé
                processor.downscale_frame(&frame, 128, 64, 64);

                // Essayer la compression pour les grandes frames
                let preview_buffer = processor.preview_buffer.clone();
                if let Some(compressed) = processor.compress_frame(&preview_buffer) {
                    let msg = WsMessage::CompressedFrame {
                        data: compressed,
                        width: 64,
                        height: 64,
                    };

                    if preview_tx
                        .send(Message::text(serde_json::to_string(&msg).unwrap()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                } else {
                    let msg = WsMessage::Frame {
                        data: processor.preview_buffer.clone(),
                    };

                    if preview_tx
                        .send(Message::text(serde_json::to_string(&msg).unwrap()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }

            // Envoyer le spectre seulement s'il a changé
            if spectrum_hash != processor.last_spectrum_hash {
                processor.last_spectrum_hash = spectrum_hash;

                // Réduire la précision pour économiser la bande passante
                let reduced_spectrum: Vec<f32> = spectrum
                    .iter()
                    .map(|&v| (v * 100.0).round() / 100.0)
                    .collect();

                let msg = WsMessage::Spectrum {
                    data: reduced_spectrum,
                };

                let _ = preview_tx
                    .send(Message::text(serde_json::to_string(&msg).unwrap()))
                    .await;
            }

            // Statistiques de débit
            if last_sent.elapsed() >= Duration::from_secs(10) {
                let fps = frame_counter as f32 / 10.0;
                println!("📊 WebSocket stats: {:.1} FPS", fps);
                last_sent = Instant::now();
                frame_counter = 0;
            }
        }

        println!("🔄 WebSocket preview thread ended");
    });

    // Thread pour envoyer les messages via WebSocket avec gestion de congestion
    let sender_handle = tokio::spawn(async move {
        let mut pending_messages: usize = 0;

        while let Some(msg) = rx.recv().await {
            // Limiter les messages en attente
            if pending_messages > 5 {
                // Sauter des frames si le client est trop lent
                continue;
            }

            pending_messages += 1;

            if ws_sender.send(msg).await.is_err() {
                break;
            }

            pending_messages = pending_messages.saturating_sub(1);
        }

        println!("🔄 WebSocket sender thread ended");
    });

    // Recevoir les commandes avec timeout
    while let Ok(msg_result) =
        tokio::time::timeout(Duration::from_secs(60), ws_receiver.next()).await
    {
        match msg_result {
            Some(Ok(msg)) => {
                if let Message::Text(text) = msg {
                    // Traiter le message de manière non-bloquante
                    let state_clone = state.clone();
                    tokio::spawn(async move {
                        process_client_message(text, state_clone);
                    });
                } else if let Message::Close(_) = msg {
                    break;
                }
            }
            Some(Err(_)) => break,
            None => break,
        }
    }

    // Cleanup
    drop(tx);
    let _ = sender_handle.await;

    println!("🔌 WebSocket connection closed");
    Ok(())
}

fn process_client_message(text: String, state: Arc<AppState>) {
    match serde_json::from_str::<WsMessage>(&text) {
        Ok(cmd) => match cmd {
            WsMessage::Effect { id } => {
                println!("🎨 Changing effect to: {}", id);
                state.effect_engine.lock().set_effect(id);
            }
            WsMessage::Param { name, value } => {
                println!("🎛️  Parameter change: {} = {}", name, value);

                match name.as_str() {
                    "colorMode" => {
                        state.effect_engine.lock().set_color_mode(&value);
                    }
                    "customColor" => {
                        let parts: Vec<f32> =
                            value.split(',').filter_map(|s| s.parse().ok()).collect();

                        if parts.len() == 3 {
                            state
                                .effect_engine
                                .lock()
                                .set_custom_color(parts[0], parts[1], parts[2]);
                        } else {
                            println!("❌ Invalid color format");
                        }
                    }
                    _ => {
                        println!("⚠️  Unknown parameter: {}", name);
                    }
                }
            }
            _ => {
                println!("⚠️  Unexpected message type");
            }
        },
        Err(e) => {
            println!("❌ Failed to parse WebSocket message: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_processor_hash() {
        let data1 = vec![1, 2, 3, 4, 5];
        let data2 = vec![1, 2, 3, 4, 5];
        let data3 = vec![1, 2, 3, 4, 6];

        let hash1 = FrameProcessor::fast_hash(&data1);
        let hash2 = FrameProcessor::fast_hash(&data2);
        let hash3 = FrameProcessor::fast_hash(&data3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_downscale() {
        let mut processor = FrameProcessor::new();
        let src = vec![255u8; 128 * 128 * 3];

        processor.downscale_frame(&src, 128, 64, 64);

        assert_eq!(processor.preview_buffer.len(), 64 * 64 * 3);
        assert!(processor.preview_buffer.iter().all(|&x| x == 255));
    }
}
