package kafka

import (
	"log"
	"time"

	kgo "github.com/segmentio/kafka-go"
)

var KafkaWriter MessageWriter
var producerConfig = ProducerConfig{
	DeliveryMode:     DeliveryModeConfirm,
	RequiredAcks:     kgo.RequireOne,
	BatchTimeoutMs:   10,
	BatchBytes:       131072,
	RequestTimeoutMs: 5000,
}

// InitKafkaProducer sets up the global Kafka writer.
func InitKafkaProducer(kafkaURL string) {
	producerConfig = LoadProducerConfigFromEnv()
	if !producerConfig.UsesKafka() {
		log.Printf("BENCHMARK_DELIVERY_MODE=%s; skipping Kafka producer initialization", producerConfig.DeliveryMode)
		KafkaWriter = nil
		return
	}

	log.Printf(
		"Initializing Kafka producer for topic 'bids' at %s (delivery_mode=%s, required_acks=%d, async=%t, batch_timeout_ms=%d, batch_bytes=%d, request_timeout_ms=%d)",
		kafkaURL,
		producerConfig.DeliveryMode,
		producerConfig.RequiredAcks,
		producerConfig.AsyncWrites(),
		producerConfig.BatchTimeoutMs,
		producerConfig.BatchBytes,
		producerConfig.RequestTimeoutMs,
	)

	KafkaWriter = &kgo.Writer{
		Addr:     kgo.TCP(kafkaURL),
		Topic:    "bids",
		Balancer: &kgo.LeastBytes{},
		Async:    producerConfig.AsyncWrites(),

		// Performance Tuning
		BatchTimeout: time.Duration(producerConfig.BatchTimeoutMs) * time.Millisecond,
		BatchSize:    65536,
		RequiredAcks: producerConfig.RequiredAcks,
		BatchBytes:   producerConfig.BatchBytes,
		ReadTimeout:  time.Duration(producerConfig.RequestTimeoutMs) * time.Millisecond,
		WriteTimeout: time.Duration(producerConfig.RequestTimeoutMs) * time.Millisecond,
	}
}

func DeliveryMode() string {
	return producerConfig.DeliveryMode
}
