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
        let mut framebytes: Vec<u8> = vec![0; 10];
        println!("e");
        stream.read_exact(&mut framebytes).unwrap();
        println!("e");
        let bits = get_bits_vec(&framebytes);
        println!("{bits}");
        let fin = bits.chars().nth(0).unwrap();
        if fin == '1' {
            break;
        }
        let payloadlength_bits = &(bits.as_str())[9..15];
        let payloadlength = u32::from_str_radix(payloadlength_bits, 2).unwrap();
        println!("{}", payloadlength);
    }
}

