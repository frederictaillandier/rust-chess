mod matchmaker;
mod tcp_connection;

use tcp_connection::ConnectionHolder;

fn main() {
    let matchmaker = matchmaker::MatchMaker::new();
    let mut connection_holder: ConnectionHolder = ConnectionHolder::new(matchmaker);
    connection_holder.start("0.0.0.0:9999");
}
