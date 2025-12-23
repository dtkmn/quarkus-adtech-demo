#!/bin/bash
# Wait for Kafka to be ready and create topics

echo "Waiting for Kafka to be ready..."
cub kafka-ready -b kafka:29092 1 60

echo "Creating Kafka topics..."
kafka-topics --bootstrap-server kafka:29092 --create --if-not-exists --topic bids --partitions 3 --replication-factor 1

echo "Kafka topics created successfully"
kafka-topics --bootstrap-server kafka:29092 --list
