use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::{error, info, instrument, Level};
use tracing_subscriber::FmtSubscriber;

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
    domain: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidApp {
    bundle: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidDevice {
    ip: Option<String>,
    os: String,
    lmt: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct BidUser {
    id: String,
}

// --- Actix Web Handler for POST /bid-request ---
#[instrument(skip_all, fields(request_id = %bid_request.id))]
async fn receive_bid(
    bid_request: web::Json<BidRequest>,
    producer: web::Data<FutureProducer>,
) -> impl Responder {
    info!("Received bid request");

    // --- STAGE 1: FAST VALIDATION ---
    if bid_request.id.is_empty() || bid_request.device.is_none() || (bid_request.site.is_none() && bid_request.app.is_none()) {
        info!("Request failed validation");
        return HttpResponse::BadRequest().json(serde_json::json!({"status": "bad request"}));
    }

    // --- STAGE 2: SIMPLE BUSINESS FILTERING ---
    if let Some(device) = &bid_request.device {
        if device.lmt == 1 {
            info!("Request filtered due to LMT=1");
            return HttpResponse::NoContent().finish();
        }
        if let Some(ip) = &device.ip {
            if ip.starts_with("10.10.") {
                info!("Request filtered due to internal IP");
                return HttpResponse::NoContent().finish();
            }
        }
    }

    // --- STAGE 3: PUSH TO KAFKA & ACKNOWLEDGE ---
    let payload = match serde_json::to_string(&bid_request.into_inner()) {
        Ok(p) => p,
        Err(_) => {
            error!("Failed to serialize request payload");
            return HttpResponse::InternalServerError().json(serde_json::json!({"status": "serialization error"}));
        }
    };

    let record: FutureRecord<String, String> = FutureRecord::to("bid_requests").payload(&payload);

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(_) => {
            info!("Successfully sent message to Kafka");
            HttpResponse::Ok().json(serde_json::json!({"status": "accepted"}))
        }
        Err(e) => {
            error!("Kafka write error: {:?}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"status": "kafka buffer full"}))
        }
    }
}

// --- Main Application Entry Point ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Get Kafka URL from environment or use a default
    let kafka_url = env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or_else(|_| "localhost:9092".to_string());
    info!("Connecting to Kafka at {}", kafka_url);

    // Create a Kafka producer
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_url)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    info!("Starting Rust AdTech Receiver on port 8080...");

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(producer.clone()))
            .route("/bid-request", web::post().to(receive_bid))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
