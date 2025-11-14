use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

// --- Data Models (mirroring the Go service) ---
#[derive(Serialize, Deserialize, Debug)]
struct BidRequest {
    id: String,
    site: Option<BidSite>,
    app: Option<BidApp>,
    device: Option<BidDevice>,
    user: Option<BidUser>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidSite {
    id: Option<String>,
    domain: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidApp {
    bundle: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidDevice {
    ip: Option<String>,
    os: Option<String>,
    lmt: i32,
    ua: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidUser {
    id: String,
}

// --- Actix Web Handler for POST /bid-request ---
async fn receive_bid(
    bid_request: web::Json<BidRequest>,
    producer: web::Data<FutureProducer>,
) -> impl Responder {
    // --- STAGE 1: FAST VALIDATION ---
    // if bid_request.id.is_empty() || bid_request.device.is_none() || (bid_request.site.is_none() && bid_request.app.is_none()) {
    if bid_request.id.is_empty() || bid_request.device.is_none() || (bid_request.site.is_none()) {
        return HttpResponse::BadRequest().json(serde_json::json!({"status": "bad request"}));
    }

    // --- STAGE 2: SIMPLE BUSINESS FILTERING ---
    if let Some(device) = &bid_request.device {
        if device.lmt == 1 {
            return HttpResponse::NoContent().finish();
        }
        if let Some(ip) = &device.ip {
            if ip.starts_with("10.10.") {
                return HttpResponse::NoContent().finish();
            }
        }
    }

    // --- STAGE 3: PUSH TO KAFKA & ACKNOWLEDGE ---f
    let payload = match serde_json::to_string(&bid_request.into_inner()) {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({"status": "serialization error"}));
        }
    };

    let record: FutureRecord<String, String> = FutureRecord::to("bids").payload(&payload);

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "accepted"})),
        Err(e) => {
            eprintln!("Kafka write error: {:?}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"status": "kafka buffer full"}))
        }
    }
}

// --- Main Application Entry Point ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get Kafka URL from environment or use a default
    let kafka_url = env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or_else(|_| "localhost:9092".to_string());
    println!("Connecting to Kafka at {}", kafka_url);

    // Create a Kafka producer
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_url)
        .set("message.timeout.ms", "5000")
        // .set("linger.ms", "200")
        .set("queue.buffering.max.messages", "1000000")
        .create()
        .expect("Producer creation error");

    println!("Starting Rust AdTech Receiver on port 8080...");

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(producer.clone()))
            .route("/bid-request", web::post().to(receive_bid))
    })
    .workers(4)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
