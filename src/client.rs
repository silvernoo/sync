use crate::crypto::{decrypt, encrypt, key_from_password};
use crate::protocol::ClipboardData;
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub async fn run(address: &str, port: u16, key: &str) -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(format!("{}:{}", address, port)).await?;
    let (mut reader, mut writer) = stream.into_split();
    let key = Arc::new(key_from_password(key));
    let clipboard = Arc::new(Mutex::new(Clipboard::new()?));
    // This mutex helps us avoid re-broadcasting a clipboard update we just received.
    let last_set_data = Arc::new(Mutex::new(None::<ClipboardData>));

    println!("Connected to server {}:{}", address, port);

    // Task to receive updates from the server
    let key_clone_read = Arc::clone(&key);
    let clipboard_clone_read = Arc::clone(&clipboard);
    let last_set_data_clone_read = Arc::clone(&last_set_data);
    let read_task = tokio::spawn(async move {
        loop {
            let mut len_bytes = [0u8; 4];
            if reader.read_exact(&mut len_bytes).await.is_err() {
                eprintln!("Connection closed by server.");
                break;
            }
            let len = u32::from_be_bytes(len_bytes) as usize;
            let mut buffer = vec![0u8; len];
            if reader.read_exact(&mut buffer).await.is_err() {
                eprintln!("Failed to read message from server.");
                break;
            }

            match decrypt(&buffer, &key_clone_read) {
                Ok(decrypted_data) => {
                    if let Ok((data, _)) = bincode::decode_from_slice::<ClipboardData, _>(&decrypted_data, bincode::config::standard()) {
                        println!("Received data from server: {:?}", data);
                        let mut clipboard = clipboard_clone_read.lock().await;
                        let mut last_set = last_set_data_clone_read.lock().await;
                        *last_set = Some(data.clone()); // Store the received data

                        match data {
                            ClipboardData::Text(text) => {
                                if let Err(e) = clipboard.set_text(text) {
                                    eprintln!("Failed to set clipboard text: {}", e);
                                }
                            }
                            ClipboardData::Image { width, height, bytes } => {
                                let img_data = ImageData {
                                    width: width as usize,
                                    height: height as usize,
                                    bytes: Cow::from(bytes),
                                };
                                if let Err(e) = clipboard.set_image(img_data) {
                                    eprintln!("Failed to set clipboard image: {}", e);
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to deserialize data from server.");
                    }
                }
                Err(_) => eprintln!("Failed to decrypt data from server."),
            }
        }
    });

    // Task to watch local clipboard and send updates
    let key_clone_write = Arc::clone(&key);
    let clipboard_clone_write = Arc::clone(&clipboard);
    let last_set_data_clone_write = Arc::clone(&last_set_data);
    let write_task = tokio::spawn(async move {
        let mut last_text = clipboard_clone_write.lock().await.get_text().ok();

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let mut clipboard = clipboard_clone_write.lock().await;
            let mut last_set = last_set_data_clone_write.lock().await;

            let current_data = match clipboard.get_text() {
                Ok(text) => {
                    if Some(&text) != last_text.as_ref() {
                        last_text = Some(text.clone());
                        Some(ClipboardData::Text(text))
                    } else {
                        None
                    }
                }
                Err(_) => match clipboard.get_image() {
                    Ok(image) => Some(ClipboardData::Image {
                        width: image.width as u32,
                        height: image.height as u32,
                        bytes: image.bytes.into_owned(),
                    }),
                    Err(_) => None,
                },
            };

            if let Some(data) = current_data {
                let should_send = match last_set.as_ref() {
                    Some(ClipboardData::Text(t)) => {
                        if let ClipboardData::Text(current_t) = &data {
                            t != current_t
                        } else { true }
                    }
                    Some(ClipboardData::Image { .. }) => {
                        // Naive check for images; we just avoid sending if it was just set.
                        // A better approach might involve hashing the image data.
                        if let ClipboardData::Image { .. } = &data {
                            false
                        } else { true }
                    }
                    None => true,
                };

                if should_send {
                     println!("Detected new clipboard data: {:?}", data);
                    match bincode::encode_to_vec(&data, bincode::config::standard()) {
                        Ok(serialized_data) => {
                            match encrypt(&serialized_data, &key_clone_write) {
                                Ok(encrypted_data) => {
                                    if writer.write_all(&(encrypted_data.len() as u32).to_be_bytes()).await.is_err() {
                                        break;
                                    }
                                    if writer.write_all(&encrypted_data).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => eprintln!("Failed to encrypt data: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to serialize data: {}", e),
                    }
                }
                // Clear the last_set data after checking to allow the same content to be sent again later.
                *last_set = None;
            }
        }
    });

    tokio::try_join!(read_task, write_task)?;

    Ok(())
}
