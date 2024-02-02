use axum::{http::HeaderMap, routing::get, Router};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
struct Config {
    #[serde(default="default_host")]
    host: String,

    #[serde(default="default_port")]
    port: u32,
}

trait ConfigHelper {
   fn get_address(&self) -> String; 
}

impl ConfigHelper for Config {
    fn get_address(&self) -> String {
        let addr_str = self.host.to_string() + ":" + &self.port.to_string();
        addr_str
    }
}

impl Config {
    const HEADER_KEYS: [&'static str; 9] = get_header_keys();
}

// Retreived from: https://github.com/pbojinov/request-ip
//
// This is not really necessary as we already know which header to use behind
// the reverse proxy used by timjebsen.com. We do it anyway in case the 
// network changes, or this app is used elsewere
//
const fn get_header_keys() -> [&'static str; 9] {
    [
        "x-client-ip",
        "x-forwarded-for",
        "cf-connecting-ip",
        "Fastly-Client-Ip",
        "True-Client-Ip",
        "X-Real-IP",
        "X-Cluster-Client-IP",
        "X-Forwarded",
        "host"
    ]
}

fn default_host() -> String {
    let host: String = "0.0.0.0".to_string();
    println!("Using default host: {}", host);
    host
}

fn default_port() -> u32 {
    let port = 80;
    println!("Using default port: {}\n\n", port);
    port
}

fn get_ip_from_headers(headers: HeaderMap) -> String {
   for key in Config::HEADER_KEYS {
        if headers.contains_key(key) {
            return headers.get(key).unwrap().to_str().unwrap().to_string(); 
        }
   };
   "! unable to retrieve ip".to_string()
}

async fn rt_return_ip(headers: HeaderMap) -> String {
    let client_ip = get_ip_from_headers(headers);
    println!("RCVD request");
    println!("returning ip: {}", client_ip);
    client_ip
}

#[tokio::main]
async fn main() {
    let config: Config = match envy::from_env::<Config>() {
       Ok(config) => config,
       Err(error) => panic!("Error: {}", error)
    };

    let app = Router::new().route("/", get(rt_return_ip));

    let listener = tokio::net::TcpListener::bind(config.get_address())
        .await
        .unwrap();
    println!("Starting server");
    axum::serve(listener, app).await.unwrap();
}

