use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use crate::{game::Game, http::Http_request, parser::{self, get_variable}, player::Player};

pub struct Server {
    listener: TcpListener,
    game: Arc<Mutex<Game>>
}

impl Server {
    pub fn new<T: std::net::ToSocketAddrs>(address: T, game: Arc<Mutex<Game>>) -> Server {
        let server = Server {
            listener: TcpListener::bind(address).unwrap(),
            game,
        };
        server
    }
    pub fn start(&self) -> JoinHandle<()> {
        let listener = Arc::new(self.listener.try_clone().expect("Failed to clone listener"));
        let game = Arc::clone(&self.game);
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

                let game = Arc::clone(&game);
                let connection_handler = thread::spawn(move || {
                    // thread::sleep(Duration::from_secs(5));
                    Server::handle_connection(stream, game);
                });
            }
        })
    }
    fn handle_connection(mut stream: TcpStream, game: Arc<Mutex<Game>>) {
        let received: String = Server::receive(&mut stream);

        let request = match Http_request::parse(&received) {
            Ok(r) => r,
            Err(s) => {
                println!("{s}");
                return;
            },
        };

        let (status_line, contents) = Server::handle_response(&request, game);

        let response = format!(
            "{}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write_all(response.as_bytes()).unwrap();
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
    fn handle_response(request: &Http_request, game: Arc<Mutex<Game>>) -> (&str, String) {
        // println!("requesting game data");
        let mut game_data = game.lock().unwrap();
        // println!("got game data access");

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
                game_data.players.push(Player::new(&username));
                // println!("list of players is {:?}", game_data.players);
            }
            else if mode == "game".to_owned() {
                let mouse = parser::get_mouse(&body_string).unwrap();
                let keys_down = parser::get_keys_down(&body_string);
                let wheel: i32 = parser::get_variable(&body_string, "wheel").unwrap().parse().unwrap();
                objects = game_data.handle_input(&username, mouse, keys_down, wheel);
            }
            else if mode == "logout".to_owned() {
                objects = game_data.logout(&username);
            }
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

