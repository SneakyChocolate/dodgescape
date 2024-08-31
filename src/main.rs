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
mod item;

use std::sync::mpsc::channel;

use server::Server;

use crate::{game::Game, server::ServerMessage};

fn main() {
    let (sms, smr) = channel::<ServerMessage>();
    let (gms, gmr) = channel::<String>();

    let game = Game::new(gms, smr);
    game.start();
    // wsl ip
    // let server = Server::new("172.28.37.92:7878", sms, gmr);
    let server = Server::new("172.19.241.59:7878", sms, gmr);
    let server_handle = server.start();
    println!("server started...");
    let _ = server_handle.join();
}

