package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/joho/godotenv"
	"github.com/segmentio/kafka-go"
)

type OriginalMessage struct {
	ID           IDWrapper `json:"id"`
	Time         string    `json:"time"`
	Action       string    `json:"action"`
	Product      IDWrapper `json:"product"`
	BeforeUpdate int       `json:"before_update"`
	AfterUpdate  int       `json:"after_update"`
}

type IDWrapper struct {
	ID IDStringWrapper `json:"id"`
}

type IDStringWrapper struct {
	String string `json:"String"`
}

type TransformedMessage struct {
	ID           string `json:"id"`
	ProductID    string `json:"productId"`
	BeforeUpdate int    `json:"before_update"`
	AfterUpdate  int    `json:"after_update"`
	Action       string `json:"action"`
	Time         string `json:"time"`
}

func main() {
	// Load environment variables
	err := godotenv.Load()
	if err != nil {
		log.Fatalf("Error loading .env file: %v", err)
	}

	// Kafka configuration
	kafkaBroker := os.Getenv("KAFKA_BROKER")
	kafkaTopic := os.Getenv("KAFKA_TOPIC")
	kafkaGroupID := os.Getenv("KAFKA_GROUP_ID")

	// Set up Kafka reader
	reader := kafka.NewReader(kafka.ReaderConfig{
		Brokers:  []string{kafkaBroker},
		GroupID:  kafkaGroupID,
		Topic:    kafkaTopic,
		MinBytes: 10e3, // 10KB
		MaxBytes: 10e6, // 10MB
	})

	// Create a channel to handle OS signals
	sigchan := make(chan os.Signal, 1)
	signal.Notify(sigchan, os.Interrupt, syscall.SIGTERM)

	// Start consuming messages
	go func() {
		for {
			m, err := reader.FetchMessage(context.Background())
			if err != nil {
				log.Printf("Error fetching message: %v", err)
				continue
			}

			rawMessage := string(m.Value)

			jsonMessage := strings.TrimPrefix(rawMessage, "Message ")

			var originalMessage OriginalMessage
			if err := json.Unmarshal([]byte(jsonMessage), &originalMessage); err != nil {
				log.Printf("Error unmarshaling message: %v", err)
				continue
			}

			transformedMessage := transformMessage(originalMessage)
			transformedMessageJSON, err := json.Marshal(transformedMessage)
			if err != nil {
				log.Printf("Error marshaling transformed message: %v", err)
				continue
			}

			log.Printf("Incoming message: %s", transformedMessageJSON)

			// Commit the message to mark it as processed
			if err := reader.CommitMessages(context.Background(), m); err != nil {
				log.Printf("Error committing message: %v", err)
			}
		}
	}()

	// Wait for a termination signal
	<-sigchan
	log.Println("Shutting down...")
	reader.Close()
}

func transformMessage(orig OriginalMessage) TransformedMessage {
	return TransformedMessage{
		ID:           orig.ID.ID.String,
		ProductID:    orig.Product.ID.String,
		BeforeUpdate: orig.BeforeUpdate,
		AfterUpdate:  orig.AfterUpdate,
		Action:       orig.Action,
		Time:         orig.Time,
	}
}
