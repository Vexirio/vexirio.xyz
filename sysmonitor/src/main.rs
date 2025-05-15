use axum::{
    http::{Method, header, HeaderValue},
    routing::get,
    Router,
    response::IntoResponse,
};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use sysinfo::{Components, Networks, System};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use procfs::process::all_processes;
use tower_http::cors::{CorsLayer, Any};

#[derive(Serialize)]
struct SystemInfo {
    total_memory: u64,
    used_memory: u64,
    networks: Vec<NetworkData>,
    components: Vec<String>,
    processes: Vec<ProcessInfo>,
}

#[derive(Serialize)]
struct NetworkData {
    interface_name: String,
    total_received: u64,
    total_transmitted: u64,
}

#[derive(Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
}

async fn get_system_info(sys: Arc<RwLock<System>>) -> SystemInfo {
    let sys = sys.read().await;
    let networks = Networks::new_with_refreshed_list();
    let components = Components::new_with_refreshed_list();

    let network_data: Vec<NetworkData> = networks
        .into_iter()
        .map(|(name, data)| NetworkData {
            interface_name: name.to_string(),
            total_received: data.total_received(),
            total_transmitted: data.total_transmitted(),
        })
        .collect();

    let components_data: Vec<String> = components
        .into_iter()
        .map(|component| format!("{:?}", component))
        .collect();

    let processes: Vec<ProcessInfo> = all_processes()
        .unwrap_or_default()
        .into_iter()
        .map(|process| ProcessInfo {
            pid: process.stat.pid as u32,
            name: process.stat.comm,
        })
        .collect();

    SystemInfo {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        networks: network_data,
        components: components_data,
        processes,
    }
}

async fn system_handler(sys: Arc<RwLock<System>>) -> impl IntoResponse {
    let info = get_system_info(sys).await;
    axum::Json(info)
}

#[tokio::main]
async fn main() {
    let sys = Arc::new(RwLock::new(System::new_all()));
    let sys_clone = sys.clone();
	let cors = CorsLayer::new()
      .allow_origin(Any) // <== Разрешаем всё (на время отладки)
  	  .allow_methods([Method::GET])
  	  .allow_headers([header::CONTENT_TYPE]);


    let app = Router::new()
        .route("/api/system", get(move || system_handler(sys_clone.clone())))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    
    // 📌 Важно: если у вас основной цикл, он должен быть в другом таске.
}

}
