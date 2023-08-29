pub struct MatchMaker {}

impl MatchMaker {
    pub fn new() -> MatchMaker {
        MatchMaker {}
    }

    pub fn on_new_player_connected(&mut self, stream: &std::net::TcpStream) {
        println!("New player connected {}", stream.peer_addr().unwrap());
    }

    pub fn on_player_says(&mut self, stream: &std::net::TcpStream, message: String) {
        println!("Player {} says {}", stream.peer_addr().unwrap(), message);
    }

    pub fn on_player_disconnected(&mut self, stream: &std::net::TcpStream) {
        println!("Player {} disconnected", stream.peer_addr().unwrap());
    }
}
