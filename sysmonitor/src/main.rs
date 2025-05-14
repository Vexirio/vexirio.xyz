use axum::{
    http::{Method, header},
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

// Сбор данных о системе
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

// Обработчик
async fn system_handler(sys: Arc<RwLock<System>>) -> impl IntoResponse {
    let info = get_system_info(sys).await;
    axum::Json(info)
}

// Основная функция
#[tokio::main]
async fn main() {
    let sys = Arc::new(RwLock::new(System::new_all()));
    let sys_clone = sys.clone();

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]);

    // Маршруты
    let app = Router::new()
        .route("/system", get(move || system_handler(sys_clone.clone())))
        .layer(cors);

    // Сервер
    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        println!("Server running at http://{}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Основной цикл
    loop {
        let mut sys = sys.write().await;
        sys.refresh_all();

        println!("=> system:");
        println!("total memory: {} bytes", sys.total_memory());
        println!("used memory : {} bytes", sys.used_memory());

        let networks = Networks::new_with_refreshed_list();
        println!("=> networks:");
        for (name, data) in &networks {
            println!("{name}: {} B down / {} B up", data.total_received(), data.total_transmitted());
        }

        let components = Components::new_with_refreshed_list();
        println!("=> components:");
        for component in &components {
            println!("{component:?}");
        }

        if let Ok(processes) = all_processes() {
            for p in processes {
                println!("PID: {}, Name: {}", p.stat.pid, p.stat.comm);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
