use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use prometheus::{Encoder, TextEncoder, IntCounter, Histogram, HistogramOpts, Registry};
use lazy_static::lazy_static;

// Prometheus metrics
lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref REQUESTS_TOTAL: IntCounter = IntCounter::new(
        "rust_receiver_requests_total",
        "Total number of requests received"
    ).expect("metric can be created");
    static ref REQUESTS_ACCEPTED: IntCounter = IntCounter::new(
        "rust_receiver_requests_accepted_total",
        "Total number of accepted requests"
    ).expect("metric can be created");
    static ref REQUESTS_REJECTED: IntCounter = IntCounter::new(
        "rust_receiver_requests_rejected_total",
        "Total number of rejected requests"
    ).expect("metric can be created");
    static ref REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "rust_receiver_request_duration_seconds",
            "Request duration in seconds"
        )
    ).expect("metric can be created");
}

fn register_metrics() {
    REGISTRY.register(Box::new(REQUESTS_TOTAL.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(REQUESTS_ACCEPTED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(REQUESTS_REJECTED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(REQUEST_DURATION.clone())).expect("collector can be registered");
}

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
    let timer = REQUEST_DURATION.start_timer();
    REQUESTS_TOTAL.inc();
    
    // --- STAGE 1: FAST VALIDATION ---
    // if bid_request.id.is_empty() || bid_request.device.is_none() || (bid_request.site.is_none() && bid_request.app.is_none()) {
    if bid_request.id.is_empty() || bid_request.device.is_none() || (bid_request.site.is_none()) {
        REQUESTS_REJECTED.inc();
        timer.observe_duration();
        return HttpResponse::BadRequest().json(serde_json::json!({"status": "bad request"}));
    }

    // --- STAGE 2: SIMPLE BUSINESS FILTERING ---
    if let Some(device) = &bid_request.device {
        if device.lmt == 1 {
            REQUESTS_REJECTED.inc();
            timer.observe_duration();
            return HttpResponse::NoContent().finish();
        }
        if let Some(ip) = &device.ip {
            if ip.starts_with("10.10.") {
                REQUESTS_REJECTED.inc();
                timer.observe_duration();
                return HttpResponse::NoContent().finish();
            }
        }
    }

    // --- STAGE 3: PUSH TO KAFKA & ACKNOWLEDGE ---f
    let payload = match serde_json::to_string(&bid_request.into_inner()) {
        Ok(p) => p,
        Err(_) => {
            REQUESTS_REJECTED.inc();
            timer.observe_duration();
            return HttpResponse::InternalServerError().json(serde_json::json!({"status": "serialization error"}));
        }
    };

    let record: FutureRecord<String, String> = FutureRecord::to("bids").payload(&payload);

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(_) => {
            REQUESTS_ACCEPTED.inc();
            timer.observe_duration();
            HttpResponse::Ok().json(serde_json::json!({"status": "accepted"}))
        }
        Err(e) => {
            eprintln!("Kafka write error: {:?}", e);
            REQUESTS_REJECTED.inc();
            timer.observe_duration();
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"status": "kafka buffer full"}))
        }
    }
}

// --- Health check handler ---
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
}

// --- Metrics endpoint handler ---
async fn metrics() -> impl Responder {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
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

    // Register Prometheus metrics
    register_metrics();
    
    println!("Starting Rust AdTech Receiver on port 8080...");

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(producer.clone()))
            .route("/bid-request", web::post().to(receive_bid))
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(metrics))
    })
    .workers(4)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
