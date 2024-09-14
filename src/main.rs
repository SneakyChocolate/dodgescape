// remember firewall inbound and outbound rules for this port
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
mod websocket;
mod bits;
mod color;
mod string;

use std::sync::mpsc::channel;

use server::Server;

use crate::{game::Game, server::ServerMessage};

fn main() {
    let (sms, smr) = channel::<ServerMessage>();

    let game = Game::new(smr);
    game.start();
    // wsl ip
    // let server = Server::new("172.28.37.92:7878", sms);
    let server = Server::new("172.19.241.59:7878", sms);
    let server_handle = server.start();
    println!("server started...");
    let _ = server_handle.join();
}

