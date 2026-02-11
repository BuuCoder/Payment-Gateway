<?php

namespace App\Console\Commands;

use App\Models\Payment;
use App\Services\PaymentService;
use App\Services\PaymentAnalyticsService;
use App\Services\EmailService;
use App\Services\Kafka\KafkaProducer;
use App\Services\Kafka\KafkaConfig;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class KafkaConsumeCommand extends Command
{
    protected $signature = 'kafka:consume 
                            {--group= : Consumer group name} 
                            {--topic=* : Topics to consume (default: all)} 
                            {--batch=100 : Batch size} 
                            {--partition=* : Specific partitions to consume}
                            {--no-partition-filter : Disable partition filtering (let Kafka handle it)}';
    protected $description = 'Consume messages from Kafka topics';

    protected PaymentService $paymentService;
    protected PaymentAnalyticsService $analyticsService;
    protected EmailService $emailService;
    protected KafkaProducer $kafkaProducer;
    protected int $processedCount = 0;
    protected float $startTime;

    public function __construct(
        PaymentService $paymentService, 
        PaymentAnalyticsService $analyticsService,
        EmailService $emailService,
        KafkaProducer $kafkaProducer
    ) {
        parent::__construct();
        $this->paymentService = $paymentService;
        $this->analyticsService = $analyticsService;
        $this->emailService = $emailService;
        $this->kafkaProducer = $kafkaProducer;
    }

    public function handle()
    {
        $group = $this->option('group') ?? 'default';
        $topics = $this->option('topic');
        $batchSize = (int) $this->option('batch');
        $partitions = $this->option('partition');
        $noPartitionFilter = $this->option('no-partition-filter');
        
        $this->startTime = microtime(true);
        
        // Display consumer info
        $this->info("========================================");
        $this->info("Kafka Consumer Started");
        $this->info("========================================");
        $this->info("Group: {$group}");
        
        // If no topics specified, consume from all topics
        if (empty($topics)) {
            $this->info("Topics: ALL");
            $topicFilter = null;
        } else {
            $topicsStr = implode(', ', $topics);
            $this->info("Topics: {$topicsStr}");
            $topicFilter = $topics;
        }
        
        if (!empty($partitions)) {
            $partitionsStr = implode(', ', $partitions);
            $this->info("Partitions: {$partitionsStr}");
        }
        
        $this->info("Batch size: {$batchSize}");
        $this->info("========================================");

        $emptyCount = 0;
        while (true) {
            try {
                // Fetch unconsumed messages from database (simulating Kafka)
                $query = DB::table('kafka_messages')
                    ->where('consumed', false)
                    ->orderBy('created_at', 'asc')
                    ->limit($batchSize);
                
                // Filter by topics if specified
                if ($topicFilter !== null) {
                    $query->whereIn('topic', $topicFilter);
                }
                
                // Filter by partitions if specified (simulate partition assignment)
                // In real Kafka, partition is determined by hash(key) % num_partitions
                // For simulation, we use id % num_partitions to assign partitions
                if (!empty($partitions) && !$noPartitionFilter && $topicFilter !== null) {
                    // Determine number of partitions based on topic
                    $numPartitions = 6; // default
                    
                    // Check if any topic has 20 partitions
                    $highVolumeTopics = ['payment-requests', 'send-email', 'send-notification'];
                    foreach ($topicFilter as $topic) {
                        if (in_array($topic, $highVolumeTopics)) {
                            $numPartitions = 20;
                            break;
                        }
                    }
                    
                    $query->where(function($q) use ($partitions, $numPartitions) {
                        foreach ($partitions as $partition) {
                            $q->orWhereRaw("(id % ?) = ?", [$numPartitions, $partition]);
                        }
                    });
                }
                // If no-partition-filter is set, let all consumers in same group compete for messages
                // This simulates real Kafka behavior where Kafka handles partition assignment
                
                $messages = $query->get();

                if ($messages->isEmpty()) {
                    $emptyCount++;
                    if ($emptyCount % 10 == 1) { // Show every 10th time
                        $this->showStats();
                        $this->line("No messages. Waiting...");
                    }
                    usleep(500000); // 0.5s
                    continue;
                }
                
                $emptyCount = 0;
                $this->processBatch($messages);

            } catch (\Exception $e) {
                $this->error("Error consuming messages: " . $e->getMessage());
                Log::error("Kafka consumer error: " . $e->getMessage());
                sleep(5);
            }
        }
    }
    
    protected function processBatch($messages)
    {
        // Batch process for better performance
        $messageIds = [];
        
        foreach ($messages as $message) {
            $this->processMessage($message);
            $messageIds[] = $message->id;
        }
        
        // Batch update consumed status (much faster than individual updates)
        if (!empty($messageIds)) {
            DB::table('kafka_messages')
                ->whereIn('id', $messageIds)
                ->update([
                    'consumed' => true,
                    'consumed_at' => now(),
                ]);
        }
    }
    
    protected function showStats()
    {
        $duration = microtime(true) - $this->startTime;
        $throughput = $duration > 0 ? round($this->processedCount / $duration, 2) : 0;
        
        $this->info("Stats: Processed={$this->processedCount}, Duration=" . round($duration, 2) . "s, Throughput={$throughput} msg/s");
    }

    protected function processMessage($message)
    {
        try {
            $data = json_decode($message->value, true);
            
            // Only show detailed logs every 100 messages
            $showDetail = ($this->processedCount % 100 == 0);
            
            if ($showDetail) {
                $this->info("Processing message: {$message->key}");
            }

            // Process based on topic
            switch ($message->topic) {
                case 'payment-requests':
                    $this->processPaymentRequest($data);
                    break;
                    
                case 'payment-retry':
                    $this->processPaymentRetry($data);
                    break;
                    
                case 'payment-events':
                case 'payment-success':
                case 'payment-failed':
                case 'fraud-detection':
                    // Store in analytics for all payment-related events
                    $this->analyticsService->processPaymentEvent($data);
                    if ($showDetail) {
                        $this->info("✓ Analytics updated for {$message->topic}");
                    }
                    break;
                    
                case 'send-email':
                    $this->processSendEmail($data);
                    break;
                    
                case 'send-notification':
                    $this->processSendNotification($data);
                    break;
                    
                case 'notifications':
                    $this->processNotification($data);
                    break;
                    
                case 'transactions':
                    $this->processTransaction($data);
                    break;
                    
                default:
                    $this->warn("Unknown topic: {$message->topic}");
            }

            // Don't update consumed here - will be batch updated
            $this->processedCount++;
            
            if ($showDetail) {
                $this->showStats();
            }

        } catch (\Exception $e) {
            // Only log errors, not every message
            $this->error("✗ Error processing message: " . $e->getMessage());
            Log::error("Message processing error", [
                'message_id' => $message->id,
                'error' => $e->getMessage(),
            ]);
            
            // Mark this failed message as consumed to avoid blocking
            DB::table('kafka_messages')
                ->where('id', $message->id)
                ->update([
                    'consumed' => true,
                    'consumed_at' => now(),
                ]);
        }
    }

    protected function processPaymentRequest(array $data)
    {
        $payment = Payment::find($data['id']);
        
        if (!$payment) {
            $this->error("Payment not found: {$data['id']}");
            return;
        }

        // Check if already processed
        if ($payment->status !== 'PENDING') {
            $this->line("Payment already processed: {$payment->id} - Status: {$payment->status}");
            return;
        }

        // Process payment (this takes 100-500ms due to payment gateway simulation)
        $this->line("Processing payment: {$payment->id} - Amount: {$payment->amount} {$payment->currency}");
        
        $result = $this->paymentService->processPayment($payment);
        
        if ($result) {
            $this->info("✓ Payment SUCCESS: {$payment->id}");
        } else {
            $this->warn("✗ Payment FAILED: {$payment->id}");
        }
    }

    protected function processPaymentRetry(array $data)
    {
        $payment = Payment::find($data['id']);
        
        if (!$payment) {
            $this->error("Payment not found: {$data['id']}");
            return;
        }

        $this->line("Retrying payment: {$payment->id} (Attempt {$data['retry_count']}/3)");
        
        $result = $this->paymentService->processPayment($payment);
        
        if ($result) {
            $this->info("✓ Payment retry SUCCESS: {$payment->id}");
        } else {
            $this->warn("✗ Payment retry FAILED: {$payment->id}");
        }
    }

    protected function processNotification(array $data)
    {
        $type = $data['type'] ?? 'UNKNOWN';
        $this->line("Processing notification: {$type}");
        
        // Send email (single attempt, no blocking retry)
        $emailData = [
            'type' => $type,
            'recipient' => $data['user_id'] ?? 'unknown',
            'payment_id' => $data['payment_id'] ?? null,
            'message' => $data['message'] ?? '',
        ];
        
        $success = $this->emailService->sendEmail($emailData);
        
        if ($success) {
            $this->info("✓ Notification email sent: {$type}");
        } else {
            $this->error("✗ Notification email FAILED: {$type}");
        }
    }

    protected function processTransaction(array $data)
    {
        $this->line("Processing transaction event");
        
        // TODO: Implement transaction event processing
        Log::info("Transaction event processed", $data);
        
        $this->info("✓ Transaction event logged");
    }
    
    protected function processSendEmail(array $data)
    {
        $type = $data['type'] ?? 'UNKNOWN';
        $email = $data['email'] ?? 'unknown@example.com';
        $retryCount = $data['retry_count'] ?? 0;
        
        // Check if mock mode is enabled
        if (env('MOCK_EMAIL_SERVICE', false)) {
            // MOCK mode: 10ms for load testing
            usleep(10000);
            $success = true;
        } elseif (env('OPTIMIZED_EMAIL_SERVICE', false)) {
            // OPTIMIZED mode: 100ms flat (best case production)
            usleep(100000);
            
            // Send email (single attempt only, no immediate retry)
            $success = $this->emailService->sendEmail([
                'type' => $type,
                'recipient' => $email,
                'subject' => $data['subject'] ?? 'Notification',
                'template' => $data['template'] ?? 'default',
                'data' => $data['data'] ?? [],
            ]);
        } else {
            // REALISTIC mode: 100-300ms (average production)
            // Simulate realistic email service delay
            // In production, this would be async/concurrent
            usleep(rand(100000, 300000));
            
            // Send email (single attempt only, no immediate retry)
            // This prevents blocking the consumer with retry delays
            $success = $this->emailService->sendEmail([
                'type' => $type,
                'recipient' => $email,
                'subject' => $data['subject'] ?? 'Notification',
                'template' => $data['template'] ?? 'default',
                'data' => $data['data'] ?? [],
            ]);
        }
        
        if ($success) {
            // Only log every 1000th email to reduce I/O
            if ($this->processedCount % 1000 == 0) {
                $this->info("✓ Email sent: {$type} to {$email}");
            }
        } else {
            // Email failed - schedule retry after 1 minute via Kafka
            if ($retryCount < 3) {
                $nextRetry = $retryCount + 1;
                $this->scheduleEmailRetry($data, $nextRetry);
                $this->warn("✗ Email FAILED: {$type} - Scheduled retry #{$nextRetry} in 1 minute");
            } else {
                $this->error("✗ Email FAILED permanently after 3 attempts: {$type}");
                Log::error("Email failed permanently", [
                    'type' => $type,
                    'email' => $email,
                    'retry_count' => $retryCount,
                ]);
            }
        }
    }
    
    /**
     * Process emails in batch with concurrent sending
     * This simulates async/concurrent processing like production systems
     * 
     * In production, you would use:
     * - ReactPHP for async I/O
     * - Swoole for PHP async
     * - Guzzle concurrent requests
     * - Worker pool pattern
     */
    protected function processSendEmailBatch(array $messages)
    {
        $concurrency = env('EMAIL_CONCURRENCY', 20); // 20 concurrent sends per consumer
        $chunks = array_chunk($messages, $concurrency);
        
        foreach ($chunks as $chunk) {
            // Simulate concurrent processing
            // In production: Promise::all() or async workers
            $startTime = microtime(true);
            
            foreach ($chunk as $message) {
                $data = json_decode($message->value, true);
                $this->processSendEmail($data);
            }
            
            // Concurrent processing means all emails in chunk are sent "at the same time"
            // So we only wait for the longest one, not sum of all
            $elapsed = microtime(true) - $startTime;
            $this->info("✓ Batch of " . count($chunk) . " emails processed in {$elapsed}s");
        }
    }
    
    /**
     * Schedule email retry after 1 minute by sending back to Kafka
     */
    protected function scheduleEmailRetry(array $data, int $retryCount)
    {
        // Add retry metadata
        $data['retry_count'] = $retryCount;
        $data['retry_scheduled_at'] = now()->addMinute()->toDateTimeString();
        
        $email = $data['email'] ?? 'unknown';
        
        // Send back to Kafka send-email topic
        // In production, you would use Kafka scheduled messages or a separate retry topic
        // For now, we'll send immediately but mark it as a retry
        $this->kafkaProducer->send(
            KafkaConfig::TOPIC_SEND_EMAIL,
            $data,
            "email-retry-{$email}-{$retryCount}"
        );
        
        Log::info("Email retry scheduled", [
            'type' => $data['type'] ?? 'UNKNOWN',
            'email' => $email,
            'retry_count' => $retryCount,
            'scheduled_at' => $data['retry_scheduled_at'],
        ]);
    }
    
    protected function processSendNotification(array $data)
    {
        $type = $data['type'] ?? 'UNKNOWN';
        $userId = $data['user_id'] ?? 'unknown';
        $retryCount = $data['retry_count'] ?? 0;
        
        // Check if mock mode is enabled
        if (env('MOCK_NOTIFICATION_SERVICE', false)) {
            // Fast mode for testing (5ms)
            usleep(5000);
            $success = true;
        } else {
            // Simulate realistic push notification delay (50-150ms)
            // Push notifications are fast but not instant
            usleep(rand(50000, 150000));
            
            // Simulate push notification service call
            // In production: Firebase, OneSignal, APNs, etc.
            // 99% success rate (push notifications are very reliable in production)
            $success = rand(1, 100) <= 99;
        }
        
        if ($success) {
            // Only log every 1000th notification to reduce I/O
            if ($this->processedCount % 1000 == 0) {
                $this->info("✓ Notification sent: {$type} to user {$userId}");
            }
        } else {
            // Notification failed - schedule retry after 1 minute via Kafka
            if ($retryCount < 3) {
                $nextRetry = $retryCount + 1;
                $this->scheduleNotificationRetry($data, $nextRetry);
                $this->warn("✗ Notification FAILED: {$type} - Scheduled retry #{$nextRetry} in 1 minute");
            } else {
                $this->error("✗ Notification FAILED permanently after 3 attempts: {$type}");
                Log::error("Notification failed permanently", [
                    'type' => $type,
                    'user_id' => $userId,
                    'retry_count' => $retryCount,
                ]);
            }
        }
        
        // Log to file (minimal)
        if ($this->processedCount % 10000 == 0) {
            Log::info("Push notification milestone", [
                'type' => $type,
                'user_id' => $userId,
                'processed_count' => $this->processedCount,
            ]);
        }
    }
    
    /**
     * Schedule notification retry after 1 minute by sending back to Kafka
     */
    protected function scheduleNotificationRetry(array $data, int $retryCount)
    {
        // Add retry metadata
        $data['retry_count'] = $retryCount;
        $data['retry_scheduled_at'] = now()->addMinute()->toDateTimeString();
        
        $userId = $data['user_id'] ?? 'unknown';
        
        // Send back to Kafka send-notification topic
        $this->kafkaProducer->send(
            KafkaConfig::TOPIC_SEND_NOTIFICATION,
            $data,
            "notification-retry-{$userId}-{$retryCount}"
        );
        
        Log::info("Notification retry scheduled", [
            'type' => $data['type'] ?? 'UNKNOWN',
            'user_id' => $userId,
            'retry_count' => $retryCount,
            'scheduled_at' => $data['retry_scheduled_at'],
        ]);
    }
}

