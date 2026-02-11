<?php

namespace App\Services\Kafka;

class KafkaConfig
{
    public static function getProducerConfig(): array
    {
        return [
            'bootstrap.servers' => env('KAFKA_BROKERS', 'localhost:9092,localhost:9094,localhost:9096'),
            'security.protocol' => env('KAFKA_SECURITY_PROTOCOL', 'PLAINTEXT'),
            'sasl.mechanisms' => env('KAFKA_SASL_MECHANISMS', ''),
            'sasl.username' => env('KAFKA_SASL_USERNAME', ''),
            'sasl.password' => env('KAFKA_SASL_PASSWORD', ''),
        ];
    }

    public static function getConsumerConfig(): array
    {
        return [
            'bootstrap.servers' => env('KAFKA_BROKERS', 'localhost:9092,localhost:9094,localhost:9096'),
            'group.id' => env('KAFKA_CONSUMER_GROUP_ID', 'fintech-banking-group'),
            'auto.offset.reset' => 'earliest',
            'enable.auto.commit' => 'false',
            'security.protocol' => env('KAFKA_SECURITY_PROTOCOL', 'PLAINTEXT'),
            'sasl.mechanisms' => env('KAFKA_SASL_MECHANISMS', ''),
            'sasl.username' => env('KAFKA_SASL_USERNAME', ''),
            'sasl.password' => env('KAFKA_SASL_PASSWORD', ''),
        ];
    }

    // Topic names
    public const TOPIC_PAYMENT_REQUESTS = 'payment-requests';
    public const TOPIC_PAYMENT_SUCCESS = 'payment-success';
    public const TOPIC_PAYMENT_FAILED = 'payment-failed';
    public const TOPIC_PAYMENT_RETRY = 'payment-retry';
    public const TOPIC_FRAUD_DETECTION = 'fraud-detection';
    public const TOPIC_NOTIFICATIONS = 'notifications';
    public const TOPIC_TRANSACTIONS = 'transactions';
    
    // New topics for email and notifications (10 partitions each)
    public const TOPIC_SEND_EMAIL = 'send-email';
    public const TOPIC_SEND_NOTIFICATION = 'send-notification';
}
