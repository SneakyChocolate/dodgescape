// remember firewall inbound and outbound rules for this port

#![allow(warnings)]

mod action;
mod bits;
mod collectable;
mod color;
mod enemy;
mod game;
mod gametraits;
mod http;
mod inventory;
mod item;
mod math;
mod parser;
mod player;
mod server;
mod spawner;
mod string;
mod vector;
mod wall;
mod websocket;

use std::sync::mpsc::channel;

use server::Server;

use crate::{game::Game, server::ServerMessage};

pub type Float = f64;

fn main() {
    let (sms, smr) = channel::<ServerMessage>();

    let game = Game::new(smr);
    game.start();
    // wsl ip
    let server = Server::new("172.28.37.92:7878", sms);
    // let server = Server::new("172.19.241.59:7878", sms);
    let server_handle = server.start();
    println!("server started...");
    let _ = server_handle.join();
}
