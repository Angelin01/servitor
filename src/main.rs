use state::AppState;
use systemd::SystemdManagerProxy;
use tokio::net::TcpListener;
use zbus::Connection;

mod controllers;
mod models;
mod systemd;
mod state;


#[tokio::main]
async fn main() {
    let dbus_conn = Connection::session().await.unwrap();
    let manager_proxy = SystemdManagerProxy::new(&dbus_conn).await.unwrap();

    let state = AppState { manager_proxy, dbus_conn };

    let app = controllers::create_router().with_state(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Servitor running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
