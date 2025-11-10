# **Quarkus Native Demo: AdTech High-Velocity Bid Receiver**

## **1\. Business Case**

In AdTech Real-Time Bidding (RTB), latency equals lost revenue. When traffic spikes, we need to spin up new instances instantly.

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

## **4\. The "Spike Test" Comparison**

We will run two versions of this exact same code:

1. **JVM Mode:** Standard OpenJDK 17 container.
2. **Native Mode:** GraalVM native executable container.

### **Test Protocol**

We don't just want to test steady state. We want to test **cold-start under fire**.

1. Stop all containers.
2. Start the load generator (e.g., k6 or hey) aiming for 500 req/sec immediately.
3. *Then* start the application container.
4. Measure:
    * Time until the first successful 200 OK response.
    * Error rate during the first 10 seconds.
    * Memory usage after 1 minute.

## **5\. Expected Results (Hypothesis)**

| Metric | Quarkus JVM | Quarkus Native | Business Impact |
| :---- | :---- | :---- | :---- |
| **Time to First OK Response** | \~2-5 seconds | \~0.05 seconds | Native captures revenue instantly. |
| **Cold Start Error Rate** | High (timeouts while warming up) | Near Zero | Native has better reliability during scaling. |
| **Memory Footprint (RSS)** | \~250MB | \~35MB | Native allows 7x more instances on the same hardware. |

