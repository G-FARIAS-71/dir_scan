use actix_web::{web, App, HttpServer, Responder};
use rand::Rng;
use get_if_addrs::get_if_addrs;
use std::net::TcpListener;
use serde::{Serialize, Deserialize};
use serde_json::json;
use walkdir::WalkDir;
use std::fs;
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
    last_modification: String,
}

#[derive(Serialize, Deserialize)]
struct ScanResult {
    path: String,
    dir: Vec<String>,
    files: Vec<FileInfo>,
}

fn scan_path_to_json(path: &str) -> serde_json::Value {
    let mut data = ScanResult {
        path: path.to_string(),
        dir: Vec::new(),
        files: Vec::new(),
    };

    for entry in WalkDir::new(path).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            data.dir.push(entry.file_name().to_string_lossy().into_owned());
        } else if metadata.is_file() {
            let last_modification = UNIX_EPOCH + metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap();
            let last_modification = format!("{}", chrono::DateTime::<chrono::Utc>::from(last_modification));
            let file_info = FileInfo {
                name: entry.file_name().to_string_lossy().into_owned(),
                size: metadata.len(),
                last_modification,
            };
            data.files.push(file_info);
        }
    }

    json!(data)
}

async fn scan_path(info: web::Path<(String,)>) -> impl Responder {
    let path = &info.0;
    let result = scan_path_to_json(path);
    serde_json::to_string_pretty(&result).unwrap()
}

fn generate_random_port() -> String {
    let mut rng = rand::thread_rng();

    loop {
        let port = rng.gen_range(1024..=65535);
        let address = format!("localhost:{}", port);

        // Tenta abrir um listener na porta gerada para checar se ela está disponível
        if TcpListener::bind(&address).is_ok() {
            return address;
        }
    }
}

fn print_localhost() {
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            println!("Interface: {}, Endereço IP: {}", iface.name, iface.addr.ip());
        }
    } else {
        println!("Não foi possível obter os endereços de interface de rede.");
    }
}

// Função principal que inicia o servidor web
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_address = generate_random_port();

    let server = HttpServer::new(|| {
        App::new()
            .route("/{path:.*}", web::get().to(scan_path))
    });

    print_localhost();
    println!("Servidor rodando em http://{}", server_address);
    server.bind(server_address)?.run().await
}
