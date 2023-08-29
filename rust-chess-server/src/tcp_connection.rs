use std::io::{ErrorKind, Read};
use std::net::TcpListener;

use std::sync::{Arc, Mutex};
use std::thread;

pub struct ConnectionHolder {}

impl ConnectionHolder {
    pub fn new() -> ConnectionHolder {
        ConnectionHolder {}
    }

    fn open_connection_thread(
        &mut self,
        ip_port: &str,
        stream_clones: Arc<Mutex<Vec<std::net::TcpStream>>>,
    ) {
        let bind_result = TcpListener::bind(ip_port);
        let listener = bind_result.expect("Failed to bind tcp connection");

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.expect("Error on incoming connection");
                stream
                    .set_nonblocking(true)
                    .expect("Failed to set non-blocking");

                let mut streams = stream_clones.lock().unwrap();
                streams.push(stream);
            }
        });
    }

    fn listen_sockets(&mut self, streams: Arc<Mutex<Vec<std::net::TcpStream>>>) {
        let mut buf = [0; 1024];
        loop {
            let mut streams = streams.lock().unwrap();
            let mut dead_stream_intexes = Vec::new();
            let mut i = 0;

            for mut stream in streams.iter() {
                let addr = stream.peer_addr().expect("didn t manage to get addr");

                buf.fill(0);
                let read_result = stream.read(&mut buf);

                match read_result {
                    Ok(byte_read) => {
                        if byte_read > 0 {
                            println!(
                                "Received bytes from client {} {} ",
                                buf.to_vec().iter().map(|b| *b as char).collect::<String>(),
                                addr
                            );
                        } else {
                            println!("Client {} disconnected", addr);
                            dead_stream_intexes.push(i);
                        }
                    }

                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            //println!("Would block");
                            //we simply ignore this case for now
                        } else {
                            println!("Error while reading from stream {}", e);
                        }
                    }
                }
                i += 1;
            }

            for index in dead_stream_intexes.iter().rev() {
                streams.remove(*index);
            }
        }
    }

    pub fn start(&mut self, ip_port: &str) {
        let streams = Arc::new(Mutex::new(Vec::new()));
        let streams_clone = streams.clone();

        self.open_connection_thread(ip_port, streams_clone);
        self.listen_sockets(streams);
    }
}
