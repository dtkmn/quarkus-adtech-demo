package main

import (
	"log"
	"os"

	"github.com/dtkmn/go-adtech-receiver/internal/handlers" // Internal import
	"github.com/dtkmn/go-adtech-receiver/internal/kafka"    // Internal import
	"github.com/gin-gonic/gin"
	metrics "github.com/zsais/go-gin-prometheus"
)

func main() {
	// Get Kafka URL from environment
	kafkaURL := os.Getenv("KAFKA_BOOTSTRAP_SERVERS")
	if kafkaURL == "" {
		kafkaURL = "localhost:9092" // Default
		log.Printf("KAFKA_BOOTSTRAP_SERVERS not set, defaulting to %s", kafkaURL)
	}

	// Initialize the Kafka Producer
	// This makes the producer available to the handlers package
	kafka.InitKafkaProducer(kafkaURL)

	// Set up the Gin server
	router := gin.Default()

	// --- Configure Prometheus Metrics ---
	m := metrics.NewPrometheus("gin")
	m.Use(router) // This makes Gin use the middleware

	// --- Define Routes ---
	// We now pass the handler function from the 'handlers' package
	router.POST("/bid-request", handlers.ReceiveBid)

	// Run the server
	log.Println("Starting Go AdTech Receiver on port 8080...")
	router.Run(":8080")
}
