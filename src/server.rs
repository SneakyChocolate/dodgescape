use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use crate::{http::Http_request, Float};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Login(String, Sender<String>),
    Logout(String),
    Input {
        name: String,
        mouse: (Float, Float),
        keys: Vec<String>,
        wheel: i32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub mode: String,
    pub username: String,
    pub x: Option<Float>,
    pub y: Option<Float>,
    pub keys_down: Option<Vec<String>>,
    pub wheel: Option<i32>,
}
impl ClientMessage {
    pub fn new(
        mode: String,
        username: String,
        x: Option<Float>,
        y: Option<Float>,
        keys_down: Option<Vec<String>>,
        wheel: Option<i32>,
    ) -> ClientMessage {
        ClientMessage {
            mode,
            username,
            x,
            y,
            keys_down,
            wheel,
        }
    }
}

pub struct Server {
    listener: TcpListener,
    sender: mpsc::Sender<ServerMessage>,
}

impl Server {
    pub fn new<T: std::net::ToSocketAddrs>(
        address: T,
        sender: mpsc::Sender<ServerMessage>,
    ) -> Server {
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
                    }
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
            }
        };
        // check if ws handshake
        let wskey = request.get_header("Sec-WebSocket-Key".to_owned());
        let mut ws = false;
        let (response, contents): (String, Vec<u8>) = match wskey {
            Some(key) => {
                ws = true;
                (crate::websocket::response(key), vec![])
            }
            None => {
                let (status_line, contents) = Self::handle_response(&request);
                (format!(
                    "{}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\n\r\n",
                    status_line,
                    contents.len()
                ), contents)
            }
        };

        let _r = stream.write_all(response.as_bytes());
        let _r = stream.write_all(&contents);
        stream.flush().unwrap();

        if !ws {
            return;
        }
        // continue if its a websocket
        crate::websocket::handle_websocket(sender, stream);
    }
    fn receive(stream: &mut TcpStream) -> String {
        let mut received: String = "".to_owned();
        // TODO loop unitl message finished reading
        let mut buffer = [0; 1024];
        let read_length = stream.read(&mut buffer).unwrap();
        // println!("{read_length}"); // prints only once without break at the end
        let actual_read_buffer = &buffer[..read_length];
        let msg = String::from_utf8(actual_read_buffer.to_vec()).unwrap();
        received.push_str(&msg);
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
            "GET /styles.css HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/styles.css").unwrap())
            }

            // ingame resources
            "GET /monocle.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/monocle.png").unwrap())
            }
            "GET /microscope.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/microscope.png").unwrap())
            }
            "GET /binoculars.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/binoculars.png").unwrap())
            }
            "GET /telescope.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/telescope.png").unwrap())
            }
            "GET /heatwave.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/heatwave.png").unwrap())
            }
            "GET /blizzard.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/blizzard.png").unwrap())
            }
            "GET /univeye.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/univeye.png").unwrap())
            }
            "GET /dragonfirerune.png HTTP/1.1" => (
                "HTTP/1.1 200 OK",
                fs::read("./res/dragonfirerune.png").unwrap(),
            ),
            "GET /hourglass.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/hourglass.png").unwrap())
            }
            "GET /orbit.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/orbit.png").unwrap()),
            "GET /blackhole.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/blackhole.png").unwrap())
            }
            "GET /push.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/push.png").unwrap()),
            "GET /speedup.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/speedup.png").unwrap())
            }
            "GET /puddle.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/puddle.png").unwrap())
            }
            "GET /heart.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/heart.png").unwrap()),

            "GET /candytop.png HTTP/1.1" => {
                ("HTTP/1.1 200 OK", fs::read("./res/candytop.png").unwrap())
            }
            // "GET /monocle.png HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read("./res/monocle.png").unwrap()),
            _ => (
                "HTTP/1.1 404 NOT FOUND",
                fs::read("./res/404.html").unwrap(),
            ),
        };

        (status_line.to_owned(), response)
    }
}

#[cfg(test)]
mod serde_test {
    use crate::{enemy::Enemy, game::DrawPack, gametraits::Radius, server::ClientMessage};

    #[test]
    fn object_with_missing_attribute() {
        let example = ClientMessage::new(
            "login".to_owned(),
            "jo; 3".to_owned(),
            Some(20.0),
            None,
            Some(vec!["KeyW".to_owned()]),
            None,
        );
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
        let example = DrawPack::new(
            "rgb(0,32,15)",
            crate::game::Shape::Circle {
                radius: Radius::Absolute(30.3),
            },
            (0.0, 0.0),
        );
        let serialized = serde_json::to_string(&example).unwrap();

        assert_eq!(serialized, "{\"color\":\"rgb(0,32,15)\",\"shape\":{\"Circle\":{\"radius\":{\"Absolute\":30.3}}},\"offset\":[0.0,0.0]}".to_owned());
    }
}
