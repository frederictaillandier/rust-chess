mod event_type;
mod matchmaker;
mod tcp_connection;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tcp_connection::ConnectionHolder;

fn main() {
    let (sender_to_tcp_connection, receiver_from_matchmaker): (
        Sender<event_type::EventType>,
        Receiver<event_type::EventType>,
    ) = mpsc::channel();
    let (sender_to_matchmaker, receiver_from_tcp_connection): (
        Sender<event_type::EventType>,
        Receiver<event_type::EventType>,
    ) = mpsc::channel();

    let matchmaker =
        matchmaker::MatchMaker::new(sender_to_tcp_connection, receiver_from_tcp_connection);
    matchmaker.start_loop_thread();
    let mut connection_holder: ConnectionHolder =
        ConnectionHolder::new(sender_to_matchmaker, receiver_from_matchmaker);
    connection_holder.start("0.0.0.0:9999");
}
