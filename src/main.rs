use std::net::SocketAddr;

mod models;
mod port;
mod server;

use port::{AttendancePort, FingerprintPort};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let port = FingerprintPort::get().expect("Unable to access the fingerprint port");

    let app = server::create_router(port).await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start the web server");
}
