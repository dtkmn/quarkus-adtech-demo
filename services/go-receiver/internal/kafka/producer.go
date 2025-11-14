package kafka

import (
	"log"
	"time"

	"github.com/segmentio/kafka-go"
)

// KafkaWriter is a package-level variable, making it accessible
// to other packages (like 'handlers') that import this one.
var KafkaWriter *kafka.Writer

// InitKafkaProducer sets up the global Kafka writer.
func InitKafkaProducer(kafkaURL string) {
	log.Printf("Initializing Kafka producer for topic 'bids' at %s", kafkaURL)

	KafkaWriter = &kafka.Writer{
		Addr:     kafka.TCP(kafkaURL),
		Topic:    "bids",
		Balancer: &kafka.LeastBytes{},
		Async:    true,

		// Performance Tuning
		BatchTimeout: 10 * time.Millisecond,
		BatchSize:    65536,
		RequiredAcks: kafka.RequiredAcks(0),
		BatchBytes:   131072, // 128 KB
	}
}
