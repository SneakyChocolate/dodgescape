mod http;
mod server;
mod parser;
mod game;
mod player;
mod enemy;
mod vector;
mod wall;
mod math;
mod action;
mod inventory;

use std::sync::{Arc, Mutex};

use server::Server;

use crate::game::Game;

fn main() {
    let game = Arc::new(Mutex::new(Game::new()));
    Game::start(&game);
    // wsl ip
    let server = Server::new("172.28.37.92:7878", Arc::clone(&game));
    // windows ip
    // let server = Server::new("192.168.178.66:7878", Arc::clone(&game));
    let server_handle = server.start();
    println!("server started...");
    let _ = server_handle.join();
}

