use tcp_connection::ConnectionHolder;

mod tcp_connection;

fn main() {
    let mut connection_holder: ConnectionHolder = ConnectionHolder::new();
    connection_holder.start("0.0.0.0:9999");
}
