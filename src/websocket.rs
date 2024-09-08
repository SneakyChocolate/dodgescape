use std::{io::Read, net::TcpStream};

use base64::prelude::*;
use sha1::{Sha1, Digest};

use crate::bits::get_bits_vec;

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
        let fin = framebytes.get(0).unwrap();
        if *fin >= 128 {
            // message is finished
            println!("finsi");
        }
        let b2 = *framebytes.get(1).unwrap();
        let length = if b2 >= 128 {
            b2 - 128
        }
        else {
            b2
        };
        println!("length: {length}");
        let mut payload = vec![0u8; length as usize];
        stream.read_exact(&mut payload);
        println!("payload: {:?}", payload);
    }
}
