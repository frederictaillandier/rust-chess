use std::net::TcpListener;
use std::net::TcpStream;

pub struct ConnectionHolder {
    streams: Vec<TcpStream>,
    addrs: Vec<std::net::SocketAddr>,
}

impl ConnectionHolder {
    pub fn new() -> ConnectionHolder {
        ConnectionHolder {
            streams: Vec::new(),
            addrs: Vec::new(),
        }
    }

    pub fn start(&mut self, ip_port: &str) {
        let bind_result = TcpListener::bind(ip_port);
        let listener = bind_result.expect("Failed to bind tcp connection");

        for stream in listener.incoming() {
            let stream = stream.expect("Error on incoming connection");
            stream
                .set_nonblocking(true)
                .expect("set_nonblocking call failed");
            let addr = stream.peer_addr().expect("failed to get peer addr");
            self.streams.push(stream);
            self.addrs.push(addr);
            self.on_connection_happen(addr);
        }
    }

    fn on_connection_happen(&mut self, addr: std::net::SocketAddr) {
        println!("Connection from {}", addr);
    }
}
