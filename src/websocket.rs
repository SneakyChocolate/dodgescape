use std::{io::{Read, Write}, net::TcpStream, sync::mpsc::channel, thread};

use base64::prelude::*;
use sha1::{Sha1, Digest};

pub fn ws_accept_key(key: &str) -> String {
    let magic_string = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key);

    // sha1
    let mut hasher = Sha1::new();
    hasher.update(magic_string.as_bytes());
    let hash = hasher.finalize();
    
    // Base64 Encoding
    BASE64_STANDARD.encode(hash)
}

pub fn response(key: &str) -> String {
    let acckey = ws_accept_key(key);
    format!("HTTP/1.1 101 Switching Protocols\nUpgrade: websocket\nConnection: Upgrade\nSec-WebSocket-Accept: {}\n\n", acckey)
}

// after handshake
pub fn handle_websocket(mut stream: TcpStream) {
    println!("ws connection established");

    let (quit_message_sender, quit_message_receiver) = channel::<()>();

    let mut send_stream = stream.try_clone().unwrap();
    let send_handle = thread::spawn(move || {
        send(&mut send_stream, "111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000".to_owned());
        loop {
            match quit_message_receiver.try_recv() {
                Ok(_) => {
                    println!("erfolgreich geschlossen");
                    break;
                },
                Err(e) => {
                    match e {
                        std::sync::mpsc::TryRecvError::Empty => {},
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            break;
                        },
                    }
                },
            }
            // send(&mut send_stream, "111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000".to_owned());
        }
    });
    let read_handle = thread::spawn(move || {
        loop {
            match read(&mut stream) {
                Ok(message) => {
                    println!("message: {message}");
                },
                Err(_) => {
                    quit_message_sender.send(()).unwrap();
                    break;
                },
            };
        }
    });

    send_handle.join().unwrap();
    read_handle.join().unwrap();
}

fn read(stream: &mut TcpStream) -> Result<String, ()> {
    let mut framebytes: Vec<u8> = vec![0; 2];
    stream.read_exact(&mut framebytes).unwrap();

    let firstbyte = framebytes.get(0).unwrap();
    let secondbyte = framebytes.get(1).unwrap();

    let opcode = firstbyte & 0x0F;
    if opcode == 0x8 {
        println!("verbindung wird geschlossen");
        return Err(());
    }

    let mut payloadlength = secondbyte & 0x7F;
    let mut extended_payloadlength = 0;


    if payloadlength == 126 {
        let mut extended_len_bytes = [0u8; 2];
        stream.read_exact(&mut extended_len_bytes).unwrap();
        extended_payloadlength = u16::from_be_bytes(extended_len_bytes) as usize;
        payloadlength = 126; // Updating it to flag 126
    }
    else if payloadlength == 127 {
        let mut extended_len_bytes = [0u8; 8];
        stream.read_exact(&mut extended_len_bytes).unwrap();
        extended_payloadlength = u64::from_be_bytes(extended_len_bytes) as usize;
    }

    let total_payload_len = if extended_payloadlength > 0 {
        extended_payloadlength
    } else {
        payloadlength as usize
    };

    // masking key
    let mut maskingkey = vec![0u8; 4];
    stream.read_exact(&mut maskingkey).unwrap();

    let mut encoded = vec![0u8; total_payload_len];
    stream.read_exact(&mut encoded).unwrap();

    let decoded = crate::websocket::decode(&encoded, &maskingkey);
    let message = String::from_utf8(decoded).unwrap();
    Ok(message)
}

fn decode(encoded: &Vec<u8>, mask: &Vec<u8>) -> Vec<u8> {
    encoded.iter()
        .enumerate()
        .map(|(i, &elt)| elt ^ mask[i % mask.len()])
        .collect()
}

fn send(stream: &mut TcpStream, message: String) {
    let message_bytes = message.as_bytes();
    let length = message_bytes.len();

    // Text-Frame and FIN flag
    stream.write_all(&[0x81]).unwrap();

    if length <= 125 {
        // Payload length for small messages
        stream.write_all(&[length as u8]).unwrap();
    } else if length <= 65535 {
        // Payload length indicator for messages between 126 and 65535
        stream.write_all(&[126]).unwrap();
        // Write length in two bytes
        stream.write_all(&[(length >> 8) as u8, (length & 0xFF) as u8]).unwrap();
    }

    // Write the message bytes
    stream.write_all(message_bytes).unwrap();
    // Ensure all data is sent
    stream.flush().unwrap();

    println!("Message sent: {}", message);
}

