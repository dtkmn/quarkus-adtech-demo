package handlers

import (
	"bytes"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/dtkmn/go-adtech-receiver/internal/kafka"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func TestReceiveBid(t *testing.T) {
	// Set Gin to Test Mode
	gin.SetMode(gin.TestMode)

	// Initialize Kafka Producer for testing
	kafka.InitKafkaProducer("localhost:9092")

	// Test case 1: Valid bid request
	t.Run("ValidBidRequest", func(t *testing.T) {
		// Setup
		router := gin.Default()
		router.POST("/bid-request", ReceiveBid)

		// Payload
		jsonStr := []byte(`{"id":"123","device":{},"app":{"bundle":"com.example.app"}}`)

		// Request
		req, _ := http.NewRequest(http.MethodPost, "/bid-request", bytes.NewBuffer(jsonStr))
		req.Header.Set("Content-Type", "application/json")

		// Response
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		// Assert
		assert.Equal(t, http.StatusOK, w.Code)
		assert.JSONEq(t, `{"status":"accepted"}`, w.Body.String())
	})

	// Test case 2: Invalid JSON
	t.Run("InvalidJSON", func(t *testing.T) {
		// Setup
		router := gin.Default()
		router.POST("/bid-request", ReceiveBid)

		// Payload
		jsonStr := []byte(`{"id":"123","device":{}}`)

		// Request
		req, _ := http.NewRequest(http.MethodPost, "/bid-request", bytes.NewBuffer(jsonStr))
		req.Header.Set("Content-Type", "application/json")

		// Response
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		// Assert
		assert.Equal(t, http.StatusBadRequest, w.Code)
		assert.JSONEq(t, `{"status":"bad request"}`, w.Body.String())
	})

	// Test case 3: LMT enabled
	t.Run("LMTEnabled", func(t *testing.T) {
		// Setup
		router := gin.Default()
		router.POST("/bid-request", ReceiveBid)

		// Payload
		jsonStr := []byte(`{"id":"123","app":{"bundle":"com.example.app"},"device":{"lmt":1}}`)

		// Request
		req, _ := http.NewRequest(http.MethodPost, "/bid-request", bytes.NewBuffer(jsonStr))
		req.Header.Set("Content-Type", "application/json")

		// Response
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		// Assert
		assert.Equal(t, http.StatusNoContent, w.Code)
	})

	// Test case 4: App and Site are nil
	t.Run("AppAndSiteNil", func(t *testing.T) {
		// Setup
		router := gin.Default()
		router.POST("/bid-request", ReceiveBid)

		// Payload
		jsonStr := []byte(`{"id":"123","device":{}}`)

		// Request
		req, _ := http.NewRequest(http.MethodPost, "/bid-request", bytes.NewBuffer(jsonStr))
		req.Header.Set("Content-Type", "application/json")

		// Response
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		// Assert
		assert.Equal(t, http.StatusBadRequest, w.Code)
		assert.JSONEq(t, `{"status":"bad request"}`, w.Body.String())
	})
}
