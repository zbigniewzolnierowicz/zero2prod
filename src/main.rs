use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run(TcpListener::bind(("127.0.0.1", 3000)).expect("Could not bind to address"))?.await
}
