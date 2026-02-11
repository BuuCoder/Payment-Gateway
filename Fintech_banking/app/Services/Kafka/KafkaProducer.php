<?php

namespace App\Services\Kafka;

use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\DB;

class KafkaProducer
{
    protected string $brokers;

    public function __construct()
    {
        $this->brokers = env('KAFKA_BROKERS', 'localhost:9092,localhost:9094,localhost:9096');
    }

    /**
     * Send message to Kafka topic
     * 
     * @param string $topic Topic name
     * @param array $message Message payload
     * @param string|null $key Partition key (should be consistent for related events)
     */
    public function send(string $topic, array $message, ?string $key = null): bool
    {
        try {
            // Generate key based on message type
            $partitionKey = $key ?? $this->generateKey($topic, $message);
            
            $payload = [
                'topic' => $topic,
                'key' => $partitionKey,
                'value' => json_encode($message),
                'timestamp' => now()->timestamp * 1000,
            ];

            // Skip logging in production/load test for performance
            if (config('app.debug') && !app()->runningInConsole()) {
                Log::info("Kafka Producer: Sending message to topic: {$topic}", [
                    'key' => $partitionKey,
                    'message' => $message
                ]);
            }

            // Store in database as fallback (simulating Kafka)
            // Use insertOrIgnore for better performance
            DB::table('kafka_messages')->insertOrIgnore([
                'topic' => $topic,
                'key' => $partitionKey,
                'value' => json_encode($message),
                'created_at' => now(),
            ]);

            return true;
        } catch (\Exception $e) {
            Log::error("Kafka Producer Error: " . $e->getMessage(), [
                'topic' => $topic,
                'message' => $message
            ]);
            return false;
        }
    }

    /**
     * Generate consistent partition key based on message content
     */
    protected function generateKey(string $topic, array $message): string
    {
        // For payment-related topics, use payment_id
        if (isset($message['id']) && str_contains($topic, 'payment')) {
            return "payment:{$message['id']}";
        }
        
        // For user-related topics, use user_id
        if (isset($message['user_id'])) {
            return "user:{$message['user_id']}";
        }
        
        // For transaction topics, use reference_number
        if (isset($message['reference_number'])) {
            return "txn:{$message['reference_number']}";
        }
        
        // Default: use timestamp-based key (not ideal for ordering)
        return "event:" . now()->timestamp;
    }

    /**
     * Send payment request
     */
    public function sendPaymentRequest(array $payment): bool
    {
        return $this->send(
            KafkaConfig::TOPIC_PAYMENT_REQUESTS,
            $payment,
            "payment:{$payment['id']}"
        );
    }

    /**
     * Send payment success
     */
    public function sendPaymentSuccess(array $payment): bool
    {
        return $this->send(
            KafkaConfig::TOPIC_PAYMENT_SUCCESS,
            $payment,
            "payment:{$payment['id']}"
        );
    }

    /**
     * Send payment failed
     */
    public function sendPaymentFailed(array $payment): bool
    {
        return $this->send(
            KafkaConfig::TOPIC_PAYMENT_FAILED,
            $payment,
            "payment:{$payment['id']}"
        );
    }

    /**
     * Send fraud detection alert
     */
    public function sendFraudAlert(array $payment): bool
    {
        return $this->send(
            KafkaConfig::TOPIC_FRAUD_DETECTION,
            $payment,
            "payment:{$payment['id']}"
        );
    }

    /**
     * Send notification
     */
    public function sendNotification(array $notification): bool
    {
        // Use user_id as key for user-related notifications
        $key = isset($notification['user_id']) 
            ? "user:{$notification['user_id']}" 
            : null;
            
        return $this->send(
            KafkaConfig::TOPIC_NOTIFICATIONS,
            $notification,
            $key
        );
    }

    /**
     * Send transaction event
     */
    public function sendTransaction(array $transaction): bool
    {
        $key = isset($transaction['reference_number'])
            ? "txn:{$transaction['reference_number']}"
            : "payment:{$transaction['payment_id']}";
            
        return $this->send(
            KafkaConfig::TOPIC_TRANSACTIONS,
            $transaction,
            $key
        );
    }
}
