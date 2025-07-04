use crate::AppState;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum WsMessage {
    Effect { id: usize },
    Param { name: String, value: String },
    Frame { data: Vec<u8> },
    Spectrum { data: Vec<f32> },
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
        println!("🌐 WebSocket server on ws://localhost:8080");

        while let Ok((stream, addr)) = listener.accept().await {
            println!("📱 New WebSocket connection from {}", addr);
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

async fn handle_connection(stream: TcpStream, state: Arc<AppState>) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Créer un canal pour envoyer des messages au WebSocket
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Thread pour envoyer les frames
    let preview_state = state.clone();
    let preview_tx = tx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(33)); // 30 FPS

        loop {
            interval.tick().await;

            // Envoyer la frame actuelle
            let frame = preview_state.led_frame.lock().clone();
            let spectrum = preview_state.spectrum.lock().clone();

            // Downscale pour preview (128x128 -> 64x64)
            let mut preview = vec![0u8; 64 * 64 * 3];
            for y in 0..64 {
                for x in 0..64 {
                    let src_idx = ((y * 2) * 128 + (x * 2)) * 3;
                    let dst_idx = (y * 64 + x) * 3;
                    preview[dst_idx..dst_idx + 3].copy_from_slice(&frame[src_idx..src_idx + 3]);
                }
            }

            let msg = WsMessage::Frame { data: preview };

            if preview_tx
                .send(Message::text(serde_json::to_string(&msg).unwrap()))
                .is_err()
            {
                break;
            }

            // Envoyer aussi le spectre
            let msg = WsMessage::Spectrum { data: spectrum };
            let _ = preview_tx.send(Message::text(serde_json::to_string(&msg).unwrap()));
        }
        println!("🔄 WebSocket preview thread started");
    });

    // Thread pour envoyer les messages via WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        println!("🔄 WebSocket sender thread started");
    });

    // Recevoir les commandes
    while let Some(msg) = ws_receiver.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                println!("📨 Received WebSocket message: {}", text);
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
                                    println!("🎨 Setting color mode to: {}", value);
                                    state.effect_engine.lock().set_color_mode(&value);
                                }
                                "customColor" => {
                                    println!("🎨 Setting custom color: {}", value);
                                    let parts: Vec<f32> =
                                        value.split(',').filter_map(|s| s.parse().ok()).collect();
                                    if parts.len() == 3 {
                                        println!(
                                            "   RGB values: R={:.2}, G={:.2}, B={:.2}",
                                            parts[0], parts[1], parts[2]
                                        );
                                        state
                                            .effect_engine
                                            .lock()
                                            .set_custom_color(parts[0], parts[1], parts[2]);
                                    } else {
                                        println!("❌ Invalid color format. Expected R,G,B but got {} parts", parts.len());
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
                        println!("   Raw message: {}", text);
                    }
                }
            }
        }
    }

    Ok(())
}
