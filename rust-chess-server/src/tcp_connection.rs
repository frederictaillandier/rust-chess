use std::io::{ErrorKind, Read};
use std::net::{TcpListener, TcpStream};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::matchmaker::MatchMaker;

pub struct ConnectionHolder {
    matchmaker: MatchMaker,
}

impl ConnectionHolder {
    pub fn new(matchmaker: MatchMaker) -> ConnectionHolder {
        ConnectionHolder {
            matchmaker: matchmaker,
        }
    }

    fn open_connection_thread(&mut self, ip_port: &str, channel_stream_sender: Sender<TcpStream>) {
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

    fn try_register_new_connections(
        &mut self,
        channel_stream_receiver: &Receiver<TcpStream>,
        streams: &mut Vec<TcpStream>,
    ) {
        let res = channel_stream_receiver.try_recv();
        match res {
            Ok(stream) => {
                self.matchmaker.on_new_player_connected(&stream);
                streams.push(stream);
            }
            Err(e) => {
                if e == mpsc::TryRecvError::Empty {
                    //println!("No new player connected");
                } else {
                    println!("Error while receiving new player {}", e);
                }
            }
        }
    }

    fn listen_sockets(&mut self, channel_stream_receiver: Receiver<TcpStream>) {
        let mut streams: Vec<TcpStream> = vec![];

        let mut buf = [0; 1024];
        loop {
            self.try_register_new_connections(&channel_stream_receiver, &mut streams);

            let mut dead_stream_intexes = Vec::new();
            let mut i = 0;

            for mut stream in streams.iter() {
                //clearing the buffer
                buf.fill(0);
                let read_result = stream.read(&mut buf);

                match read_result {
                    Ok(byte_read) => {
                        if byte_read > 0 {
                            //received something
                            self.matchmaker.on_player_says(
                                stream,
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
                self.matchmaker.on_player_disconnected(&streams[*index]);
                streams.remove(*index);
            }
        }
    }

    pub fn start(&mut self, ip_port: &str) {
        let (channel_stream_sender, channel_stream_receiver): (
            Sender<TcpStream>,
            Receiver<TcpStream>,
        ) = mpsc::channel();
        self.open_connection_thread(ip_port, channel_stream_sender);
        self.listen_sockets(channel_stream_receiver);
    }
}
