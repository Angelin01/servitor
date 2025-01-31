use axum::{extract::State, routing::{get, post}, Router, extract::Path, Json};
use serde::Serialize;
use tokio::net::TcpListener;
use zbus::{Connection, proxy};
use crate::manager::SystemdManagerProxy;
use crate::unit::SystemdUnitProxy;

mod manager;
mod unit;

#[derive(Clone)]
struct AppState {
    manager_proxy: SystemdManagerProxy<'static>,
    conn: Connection,
}

#[derive(Serialize)]
struct ServiceResponse {
    service: String,
    status: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn start_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.start_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "starting".into() })
}

async fn stop_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.stop_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "stopping".into() })
}

async fn restart_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.restart_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "restarting".into() })
}

async fn status_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    let unit_path = state.manager_proxy.get_unit(&service.as_str()).await.unwrap();
    let unit_proxy = SystemdUnitProxy::new(&state.conn, unit_path.to_string()).await.unwrap();
    let status = unit_proxy.active_state().await.unwrap();
    Json(ServiceResponse { service: unit_path.to_string(), status: status })
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse { status: "OK".into() })
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/services/{service}/start", post(start_service))
        .route("/api/v1/services/{service}/stop", post(stop_service))
        .route("/api/v1/services/{service}/restart", post(restart_service))
        .route("/api/v1/services/{service}/status", get(status_service))
        .route("/health", get(health_check))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let conn = Connection::session().await.unwrap();
    let manager_proxy = SystemdManagerProxy::new(&conn).await.unwrap();

    let state = AppState { manager_proxy, conn };

    let app = create_router(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Servitor running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
