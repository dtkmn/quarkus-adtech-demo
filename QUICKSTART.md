# Quick Start Guide - Kafka KRaft Edition

## Prerequisites
- Docker Desktop running
- 8GB+ RAM available
- Ports available: 8070-8074, 9090, 9092, 3000, 5432, 9000

## Testing with Docker Compose

### 1. Clean Start
```bash
# Make sure Docker Desktop is running
docker ps

# Stop any existing containers
docker-compose down -v

# Kill any port conflicts
lsof -ti:9090,3000,16686 | xargs kill -9 2>/dev/null || true
```

### 2. Start Infrastructure First (Test KRaft)
```bash
# Start just Kafka to verify KRaft works
docker-compose up -d kafka

# Wait for Kafka to be ready (30-40 seconds)
sleep 40

# Verify Kafka is running in KRaft mode
docker-compose logs kafka | grep "KRaft"
# Should see: "Running in KRaft mode..."

# Check topic was created
docker-compose exec kafka kafka-topics --bootstrap-server localhost:9092 --list
# Should see: bids
```

### 3. Start Full Stack
```bash
# Start everything
docker-compose up -d

# Wait for all services to be healthy (60 seconds)
sleep 60

# Check status
docker-compose ps
```

### 4. Quick Test
```bash
# Test Quarkus Receiver
curl -X POST http://localhost:8070/bid-request \
  -H "Content-Type: application/json" \
  -d '{
    "id":"test-1",
    "impressionId":"imp-1",
    "price":1.5,
    "timestamp":"2024-11-18T08:00:00Z",
    "site":{"id":"site123"},
    "device":{"ip":"192.168.1.1","lmt":0}
  }'

# Should return: {"status":"accepted"}
```

### 5. Verify Data Flow
```bash
# Check messages in Kafka
docker-compose exec kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic bids \
  --from-beginning \
  --max-messages 1 \
  --timeout-ms 5000

# Check database (after ~5 seconds)
docker-compose exec postgres psql -U user -d postgres \
  -c "SELECT COUNT(*) FROM bid_records;"
```

## Automated Test

Run the comprehensive test script:

```bash
chmod +x scripts/test-kraft-stack.sh
./scripts/test-kraft-stack.sh
```

This will:
- ✅ Start all services
- ✅ Test all 4 receivers (Quarkus JVM, Native, Go, Rust)
- ✅ Verify Kafka messages
- ✅ Check database persistence
- ✅ Show monitoring URLs

## Access Monitoring

Once everything is running:

- **Kafdrop** (Kafka UI): http://localhost:9000
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000
- **PostgreSQL**: localhost:5432 (user/password: user/password)

## Performance Testing

```bash
# Load test with k6
k6 run --vus 100 --duration 30s k6/load-test.js

# Or test different receivers
BASE_URL=http://localhost:8072 k6 run k6/load-test.js  # Go
BASE_URL=http://localhost:8073 k6 run k6/load-test.js  # Rust
```

## Troubleshooting

### Port Already Allocated
```bash
# Find and kill process using the port
lsof -ti:9090 | xargs kill -9
```

### Kafka Not Starting
```bash
# Check logs
docker-compose logs kafka

# Look for "Running in KRaft mode..."
# If you see ZooKeeper errors, you're using old config
```

### Services Not Healthy
```bash
# Check individual service logs
docker-compose logs quarkus-receiver
docker-compose logs kafka

# Restart specific service
docker-compose restart quarkus-receiver
```

### No Messages in Database
```bash
# Check if sinker is running
docker-compose logs quarkus-sinker

# Check if topic exists
docker-compose exec kafka kafka-topics --bootstrap-server localhost:9092 --list
```

## Clean Up

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (fresh start)
docker-compose down -v
```

## Next Steps

Once docker-compose works:
1. Deploy to Kubernetes: See [KUBERNETES.md](helm/KUBERNETES.md)
2. Set up GitOps: See [argocd/README.md](argocd/README.md)
3. Performance tuning: Adjust replicas in Helm values.yaml

## KRaft Benefits

Compared to the old ZooKeeper setup:
- ✅ One less container (Kafka only, no ZooKeeper)
- ✅ 50% less memory (512MB-1GB vs 2-4GB)
- ✅ 30% faster startup
- ✅ Simpler architecture
- ✅ Future-proof (ZooKeeper deprecated in Kafka 3.5+)
