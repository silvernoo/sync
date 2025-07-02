use crate::crypto::{decrypt, key_from_password};
use crate::protocol::ClipboardData;
use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

pub async fn run(address: &str, port: u16, key: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("{}:{}", address, port)).await?;
    let key = Arc::new(key_from_password(key));
    let (tx, _) = broadcast::channel::<(Vec<u8>, std::net::SocketAddr)>(16);

    println!("Server listening on {}:{}", address, port);

    loop {
        let (stream, addr) = listener.accept().await?;
        let key = Arc::clone(&key);
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            println!("Client connected: {}", addr);
            let (mut reader, mut writer) = stream.into_split();

            // Two tasks for each client: one for reading from the client and broadcasting,
            // and one for receiving from the broadcast and writing to the client.
            let key_clone = Arc::clone(&key);
            let read_task = async move {
                loop {
                    let mut len_bytes = [0u8; 4];
                    if reader.read_exact(&mut len_bytes).await.is_err() {
                        break;
                    }
                    let len = u32::from_be_bytes(len_bytes) as usize;
                    let mut buffer = vec![0u8; len];
                    if reader.read_exact(&mut buffer).await.is_err() {
                        break;
                    }

                    // Decrypt and deserialize to check data integrity
                    match decrypt(&buffer, &key_clone) {
                        Ok(decrypted_data) => {
                            if let Ok((clipboard_data, _)) = bincode::decode_from_slice::<ClipboardData, _>(&decrypted_data, bincode::config::standard()) {
                                println!("Received data from {}: {:?}", addr, clipboard_data);
                                // Broadcast the still-encrypted data
                                if tx.send((buffer, addr)).is_err() {
                                    eprintln!("Failed to broadcast message");
                                }
                            } else {
                                eprintln!("Failed to deserialize data from {}", addr);
                            }
                        }
                        Err(_) => eprintln!("Failed to decrypt data from {}", addr),
                    }
                }
                println!("Client disconnected: {}", addr);
            };

            let write_task = async move {
                loop {
                    match rx.recv().await {
                        Ok((msg, sender_addr)) => {
                            // Don't send the message back to the original sender
                            if sender_addr != addr {
                                if writer.write_all(&(msg.len() as u32).to_be_bytes()).await.is_err() {
                                    break;
                                }
                                if writer.write_all(&msg).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(_) => break, // Channel closed
                    }
                }
            };

            tokio::select! {
                _ = read_task => {},
                _ = write_task => {},
            }
        });
    }
}