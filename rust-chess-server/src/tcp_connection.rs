use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::event_type;

struct Player {
    uid: u32,
    stream: TcpStream,
}

pub struct ConnectionHolder {
    connection_counter: u32,
    players: Vec<Player>,
    sender_to_matchmaker: Sender<event_type::EventType>,
    receiver_from_matchmaker: Receiver<event_type::EventType>,
}

impl ConnectionHolder {
    pub fn new(
        sender_to_matchmaker: Sender<event_type::EventType>,
        receiver_from_matchmaker: Receiver<event_type::EventType>,
    ) -> ConnectionHolder {
        ConnectionHolder {
            connection_counter: 0,
            players: vec![],
            sender_to_matchmaker: sender_to_matchmaker,
            receiver_from_matchmaker: receiver_from_matchmaker,
        }
    }

    fn open_connection_loop_thread(
        &mut self,
        ip_port: &str,
        channel_stream_sender: Sender<TcpStream>,
    ) {
        let bind_result = TcpListener::bind(ip_port);
        let listener = bind_result.expect("Failed to bind tcp connection");

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.expect("Error on incoming connection");
                stream
                    .set_nonblocking(true)
                    .expect("Failed to set non-blocking");
                channel_stream_sender.send(stream).unwrap();
            }
        });
    }

    fn try_register_new_connections(&mut self, channel_stream_receiver: &Receiver<TcpStream>) {
        let res = channel_stream_receiver.try_recv();
        match res {
            Ok(stream) => {
                let uid = self.connection_counter;
                self.connection_counter += 1;
                self.sender_to_matchmaker
                    .send(event_type::EventType::PlayerConnect(uid))
                    .unwrap();
                self.players.push(Player { uid, stream });
            }
            Err(e) => {
                if e == mpsc::TryRecvError::Empty {
                    //Nothing happened
                } else {
                    println!("Error while receiving new player {}", e);
                }
            }
        }
    }

    fn read_sockets_loop(&mut self, channel_stream_receiver: Receiver<TcpStream>) {
        let mut buf = [0; 1024];
        loop {
            self.try_register_new_connections(&channel_stream_receiver);

            let mut dead_stream_intexes = Vec::new();
            let mut i = 0;

            for player in self.players.iter() {
                //clearing the buffer
                buf.fill(0);

                //Unfortunately it looks like we can't use the stream directly, we need to clone it
                let read_result = player.stream.try_clone().unwrap().read(&mut buf);

                match read_result {
                    Ok(byte_read) => {
                        if byte_read > 0 {
                            //received something
                            self.sender_to_matchmaker
                                .send(event_type::EventType::PlayerSay(
                                    player.uid,
                                    buf.to_vec().iter().map(|b| *b as char).collect::<String>(),
                                ))
                                .unwrap();
                        } else {
                            // Client disconnected
                            dead_stream_intexes.push(i);
                        }
                    }

                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            // Nothing received
                        } else {
                            println!("Error while reading from stream {}", e);
                        }
                    }
                }
                i += 1;
            }

            for index in dead_stream_intexes.iter().rev() {
                self.sender_to_matchmaker
                    .send(event_type::EventType::PlayerDisconnect(
                        self.players[*index].uid,
                    ))
                    .unwrap();

                self.players.remove(*index);
            }
        }
    }

    pub fn say_to_player(&mut self, player_uid: u32, message: String) {
        let player = self.players.iter_mut().find(|p| p.uid == player_uid);
        match player {
            Some(player) => {
                player.stream.write(message.as_bytes());
            }
            None => {
                println!("Player {} not found", player_uid);
            }
        }
    }

    pub fn start(&mut self, ip_port: &str) {
        let (channel_stream_sender, channel_stream_receiver): (
            Sender<TcpStream>,
            Receiver<TcpStream>,
        ) = mpsc::channel();
        self.open_connection_loop_thread(ip_port, channel_stream_sender);
        self.read_sockets_loop(channel_stream_receiver);
    }
}
