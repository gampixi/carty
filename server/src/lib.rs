mod dispatcher;
mod server;
mod room;
mod client;
mod magic;

use dispatcher::Dispatcher;
use server::Server;

pub fn run(address: &str) {
    let server = Server::new();
    let dispatcher = Dispatcher::new(address, server);

    dispatcher.handle_requests();
}