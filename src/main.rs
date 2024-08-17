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
mod gametraits;
mod collectable;

use std::sync::mpsc::channel;

use server::Server;

use crate::{game::Game, server::ServerMessage};

fn main() {
    let (sms, smr) = channel::<ServerMessage>();
    let (gms, gmr) = channel::<String>();

    let game = Game::new(gms, smr);
    game.start();
    // wsl ip
    let server = Server::new("172.28.37.92:7878", sms, gmr);
    // windows ip
    // let server = Server::new("192.168.178.66:7878", Arc::clone(&game));
    let server_handle = server.start();
    println!("server started...");
    let _ = server_handle.join();
}

