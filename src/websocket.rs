use std::{io::{Read, Write}, net::TcpStream};

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
    loop {
        let mut framebytes: Vec<u8> = vec![0; 2];
        stream.read_exact(&mut framebytes).unwrap();

        let firstbyte = framebytes.get(0).unwrap();
        let secondbyte = framebytes.get(1).unwrap();

        if *firstbyte >= 128 {
        }
        let opcode = firstbyte & 0x0F;
        if opcode == 0x8 {
            println!("verbindung wird geschlossen");
            break;
        }

        let mut payloadlength = secondbyte & 0x7F;
        println!("length: {payloadlength}");
        if payloadlength == 126 {
            // handle next byte
            println!("------------");
        }
        // masking key
        let mut maskingkey = vec![0u8; 4];
        stream.read_exact(&mut maskingkey);

        let mut encoded = vec![0u8; payloadlength as usize];
        stream.read_exact(&mut encoded);

        let decoded = crate::websocket::decode(&encoded, &maskingkey);
        let message = String::from_utf8(decoded);
        println!("{:?}", message);
        send(&mut stream, "111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000111111111112222222222244444444444555555555556666666666677777777777888888888889999999999900000000000".to_owned());
    }
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

    stream.write_all(&[0x81]); // Text-Frame and FIN flag

    if length <= 125 {
        stream.write_all(&[length as u8]); // Payload length for small messages
    } else if length <= 65535 {
        stream.write_all(&[126]); // Payload length indicator for messages between 126 and 65535
        stream.write_all(&[(length >> 8) as u8, (length & 0xFF) as u8]); // Write length in two bytes
    }

    stream.write_all(message_bytes); // Write the message bytes
    stream.flush(); // Ensure all data is sent

    println!("Message sent: {}", message);
}

