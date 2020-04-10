mod dispatcher;
mod server;
mod room;
mod client;
mod magic;
mod actions;

use dispatcher::Dispatcher;
use server::Server;

pub fn run(address: &str) {
    let server = Server::new();
    let dispatcher = Dispatcher::new(address, server);
    println!("Listening on {}", address);

    dispatcher.handle_requests();
}