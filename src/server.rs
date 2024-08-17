use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread::{self, JoinHandle}};

use crate::{game::Game, http::Http_request, parser::{self, get_variable}, player::Player};

#[derive(Debug)]
pub enum ServerMessage {
    Login(String),
    Logout(String),
    Input{name: String, mouse: (f32, f32), keys: Vec<String>, wheel: i32 },
}

pub struct Server {
    listener: TcpListener,
    sender: mpsc::Sender<ServerMessage>,
    receiver: mpsc::Receiver<String>,
}

impl Server {
    pub fn new<T: std::net::ToSocketAddrs>(address: T, sender: mpsc::Sender<ServerMessage>, receiver: mpsc::Receiver<String>) -> Server {
        let server = Server {
            listener: TcpListener::bind(address).unwrap(),
            sender,
            receiver,
        };
        server
    }
    pub fn start(mut self) -> JoinHandle<()> {
        let listener = Arc::new(self.listener.try_clone().expect("Failed to clone listener"));
        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(result) => result,
                    Err(_) => {
                        println!("connection canceled");
                        continue;
                    },
                };
                // println!("conntection incoming");

                self.handle_connection(stream);
            }
        })
    }
    fn handle_connection(&mut self, mut stream: TcpStream) {
        let received: String = Server::receive(&mut stream);

        let request = match Http_request::parse(&received) {
            Ok(r) => r,
            Err(s) => {
                println!("{s}");
                return;
            },
        };

        let (status_line, contents) = self.handle_response(&request);

        let response = format!(
            "{}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        // stream.write_all(response.as_bytes()).unwrap();
        match stream.write_all(response.as_bytes()) {
            Ok(_) => {},
            Err(_) => {},
        };
        stream.flush().unwrap();
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
    fn handle_response(&mut self, request: &Http_request) -> (&str, String) {
        let body_string = request.body.join("\n");
        // println!("received: {:#?}", body_string);

        // parsing
        let mode_option = get_variable(&body_string, "mode");
        let mut objects = "".to_owned();

        // if post requeset is normal with mode
        // handle mode
        if let Some(mode) = mode_option {
            let username = parser::get_variable(&body_string, "username").unwrap();
            if mode == "login".to_owned() {
                self.sender.send(ServerMessage::Login(username));
            }
            else if mode == "game".to_owned() {
                let mouse = parser::get_mouse(&body_string).unwrap();
                let keys_down = parser::get_keys_down(&body_string);
                let wheel: i32 = parser::get_variable(&body_string, "wheel").unwrap().parse().unwrap();
                self.sender.send(ServerMessage::Input { name: username, mouse , keys: keys_down, wheel });
            }
            else if mode == "logout".to_owned() {
                self.sender.send(ServerMessage::Logout(username));
            }
            objects = self.receiver.recv().unwrap();
        }

        // getting the output
        let (status_line, response) = match request.request_line.as_str() {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
            "POST / HTTP/1.1" => ("HTTP/1.1 200 OK", objects),
            "OPTIONS / HTTP/1.1" => ("HTTP/1.1 200 OK", "".to_owned()),
            _ => ("HTTP/1.1 404 NOT FOUND", fs::read_to_string("404.html").unwrap()),
        };

        (status_line, response)
    }
}

