use std::io::{ErrorKind, Read};
use std::net::{TcpListener, TcpStream};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::matchmaker::MatchMaker;

struct Player {
    uid: u32,
    stream: TcpStream,
}

pub struct ConnectionHolder {
    matchmaker: MatchMaker,
    connection_counter: u32,
    players: Vec<Player>,
}

impl ConnectionHolder {
    pub fn new(matchmaker: MatchMaker) -> ConnectionHolder {
        ConnectionHolder {
            matchmaker: matchmaker,
            connection_counter: 0,
            players: vec![],
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
                self.matchmaker.on_new_player_connected(uid);
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
                            self.matchmaker.on_player_says(
                                player.uid,
                                buf.to_vec().iter().map(|b| *b as char).collect::<String>(),
                            );
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
                self.matchmaker
                    .on_player_disconnected(self.players[*index].uid);
                self.players.remove(*index);
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
