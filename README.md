# **Quarkus Native Demo: High-Velocity Bid Receiver**

## **1\. Business Case**

In Real-Time Bidding (RTB), latency equals lost revenue. When traffic spikes, we need to spin up new instances instantly.

* **Traditional Java (JVM):** Takes 5-15 seconds to start and "warm up" (JIT compilation). We miss thousands of bid requests during this window.
* **Quarkus Native:** Starts in \<50ms, fully ready to handle peak load. Zero missed revenue during scale-out events.

## **2\. The Scenario**

This service acts as the "Frontline Receiver" for RTB requests.

1. Receives a high-volume stream of POST /bid-request (JSON payloads).
2. Performs fast validation (is the payload malformed? is the device blocked?).
3. Pushes valid requests to a Kafka topic for the complex Decision Engine to process.

## **3\. Real-World Data Sample (OpenRTB Simplified)**

We will perform load testing using realistic, albeit simplified, OpenRTB JSON payloads.

```json
{
  "id": "80ce30c53c16e6ede735f123ef6e32361bfc7b22",
  "at": 1,
  "cur": ["USD"],
  "imp": [
    {
      "id": "1",
      "banner": { "w": 300, "h": 250, "pos": 1 }
    }
  ],
  "site": {
    "id": "102855",
    "domain": "espn.com",
    "cat": ["IAB17"]
  },
  "device": {
    "ua": "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)...",
    "ip": "123.145.167.10",
    "os": "iOS",
    "devicetype": 1
  },
  "user": {
    "id": "55816b39711f9b5acf3b90e313ed29e51665623f"
  }
}
```

## 4. High-Level Architecture

The system is designed for high-throughput and real-time processing of bid requests. It consists of multiple, interchangeable receiver services that perform initial validation and then push the data to a Kafka topic for more complex, asynchronous processing by a "sinker" or "decision engine."

```
+----------------+      +----------------+      +----------------+
|                |      |                |      |                |
|  Quarkus       |      |  Go Receiver   |      |  Rust Receiver |
|  Receiver      |      |                |      |                |
| (Port 8071)    |      |  (Port 8072)   |      |  (Port 8073)   |
|                |      |                |      |                |
+-------+--------+      +--------+-------+      +--------+-------+
        |                        |                        |
        |  POST /bid-request     |  POST /bid-request     |  POST /bid-request
        |                        |                        |
        v                        v                        v
+----------------------------------------------------------------+
|                                                                |
|                         Kafka Topic                            |
|                       ("bid_requests")                         |
|                                                                |
+----------------------------------------------------------------+
                         |
                         |  (Consumed by Sinker)
                         |
                         v
+------------------------+-----------------------+
|                                                |
|               Quarkus Sinker                   |
|          (Complex Decision Engine)             |
|                                                |
+------------------------------------------------+
```

[//]: # (## **4\. The "Spike Test" Comparison**)


## **5\. Comprehensive Benchmark Comparison**

The following table compares our actual test results with performance for traditional stacks in a similar Docker Desktop environment (constrained to \~3-4 vCPUs).

| Metric | Go (Gin) | Quarkus Native | Rust (Actix) | Spring Boot (JVM)* | Python (FastAPI)* |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **Max Throughput** | **~31,000 RPS** | **~30,000 RPS** | **TBD** | ~14,000 RPS | ~7,000 RPS |
| **Avg. Latency** | **1.57ms** | **1.70ms** | **TBD** | ~8.5ms | ~25ms |
| **Idle Memory** | **~15 MB** | **~35 MB** | **TBD** | ~450 MB | ~120 MB |
| **Startup Time** | **Instant** | **0.05s** | **TBD** | 10s+ | 1s |
| **Bottleneck** | Network/Infra | Network/Infra | **TBD** | CPU (JIT Warmup) | CPU (GIL) |


**Key Takeaways:**

1. **Go & Quarkus Native** are in a league of their own. They are so fast they saturate the Docker network (\~30k RPS) before their own code becomes the bottleneck.
2. **Spring Boot (Standard JVM)** is robust but heavy. It requires significantly more memory (\~10x) and takes seconds to start, making it less ideal for serverless or instant-scaling AdTech scenarios.
3. **Python (FastAPI)** is excellent for development speed but struggles with raw throughput in high-concurrency scenarios due to the Global Interpreter Lock (GIL) and interpreter overhead. To match Go's 30k RPS, you would likely need 4-5x more hardware.
