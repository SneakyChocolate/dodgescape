use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::mpsc::{self, Sender}, thread::{self, JoinHandle}};

use crate::http::Http_request;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Login(String, Sender<String>),
    Logout(String),
    Input{name: String, mouse: (f32, f32), keys: Vec<String>, wheel: i32},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub mode: String,
    pub username: String,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub keys_down: Option<Vec<String>>,
    pub wheel: Option<i32>,
}
impl ClientMessage {
    pub fn new(mode: String, username: String, x: Option<f32>, y: Option<f32>, keys_down: Option<Vec<String>>, wheel: Option<i32>) -> ClientMessage {
        ClientMessage { mode, username, x, y, keys_down, wheel }
    }
}

pub struct Server {
    listener: TcpListener,
    sender: mpsc::Sender<ServerMessage>,
}

impl Server {
    pub fn new<T: std::net::ToSocketAddrs>(address: T, sender: mpsc::Sender<ServerMessage>) -> Server {
        let server = Server {
            listener: TcpListener::bind(address).unwrap(),
            sender,
        };
        server
    }
    pub fn start(self) -> JoinHandle<()> {
        thread::spawn(move || {
            for stream in self.listener.incoming() {
                let stream = match stream {
                    Ok(result) => result,
                    Err(_) => {
                        println!("connection canceled");
                        continue;
                    },
                };
                // println!("conntection incoming");
                let sender = self.sender.clone();

                thread::spawn(move || {
                    Self::handle_connection(sender, stream);
                });
            }
        })
    }
    fn handle_connection(sender: mpsc::Sender<ServerMessage>, mut stream: TcpStream) {
        let received: String = Server::receive(&mut stream);

        let request = match Http_request::parse(&received) {
            Ok(r) => r,
            Err(s) => {
                println!("{s}");
                return;
            },
        };
        // check if ws handshake
        let wskey = request.get_header("Sec-WebSocket-Key".to_owned());
        let mut ws = false;
        let (response, contents): (String, Vec<u8>) = match wskey {
            Some(key) => {
                ws = true;
                (crate::websocket::response(key), vec![])
            },
            None => {
                let (status_line, contents) = Self::handle_response(&request);
                (format!(
                    "{}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\n\r\n",
                    status_line,
                    contents.len()
                ), contents)
            },
        };

        let _r = stream.write_all(response.as_bytes());
        let _r = stream.write_all(&contents);
        stream.flush().unwrap();

        if !ws {return;}
        // continue if its a websocket
        crate::websocket::handle_websocket(sender, stream);
    }
    fn receive(stream: &mut TcpStream) -> String {
        let mut received: String = "".to_owned();
        // TODO fix this loop until message finished
        loop {
            let mut buffer = [0; 1024];
            let read_length = stream.read(&mut buffer).unwrap();
            // println!("{read_length}"); // prints only once without break at the end
            if read_length <= 0 {
                break;
            }
            let actual_read_buffer = &buffer[..read_length];
            let msg = String::from_utf8(actual_read_buffer.to_vec()).unwrap();
            received.push_str(&msg);
            break; // added break so it works
        }
        received
    }
    fn handle_response(request: &Http_request) -> (String, Vec<u8>) {

        // getting the output
        let (status_line, response): (&str, Vec<u8>) = match request.request_line.as_str() {
            // "POST / HTTP/1.1" => ("HTTP/1.1 200 OK", objects.into()),
            "OPTIONS / HTTP/1.1" => ("HTTP/1.1 200 OK", "".to_owned().into()),

            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/hello.html").unwrap()),
            "GET /bg.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/bg.png").unwrap()),
            "GET /icon.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/icon.png").unwrap()),
            "GET /script.js HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/script.js").unwrap()),
            "GET /styles.css HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/styles.css").unwrap()),
            _ => ("HTTP/1.1 404 NOT FOUND", fs::read("./res/404.html").unwrap()),
        };

        (status_line.to_owned(), response)
    }
}


#[cfg(test)]
mod serde_test {
    use crate::{enemy::Enemy, game::DrawPack, gametraits::Radius, server::ClientMessage};

    #[test]
    fn object_with_missing_attribute() {
        let example = ClientMessage::new("login".to_owned(), "jo; 3".to_owned(), Some(20.0), None, Some(vec!["KeyW".to_owned()]), None);
        let serialized = serde_json::to_string(&example).unwrap();

        assert_eq!(serialized, "{\"mode\":\"login\",\"username\":\"jo; 3\",\"x\":20.0,\"y\":null,\"keys_down\":[\"KeyW\"],\"wheel\":null}".to_owned());
    }

    #[test]
    fn new_radius_enum() {
        let example = Radius::Relative(5.0);
        let serialized = serde_json::to_string(&example).unwrap();

        assert_eq!(serialized, "{\"Relative\":5.0}".to_owned());
    }
    #[test]
    fn new_radius_enum_dp() {
        let example = DrawPack::new("rgb(0,32,15)", crate::game::Shape::Circle { radius: Radius::Absolute(30.3) }, (0.0, 0.0));
        let serialized = serde_json::to_string(&example).unwrap();

        assert_eq!(serialized, "{\"color\":\"rgb(0,32,15)\",\"shape\":{\"Circle\":{\"radius\":{\"Absolute\":30.3}}},\"offset\":[0.0,0.0]}".to_owned());
    }
}
