<?php

namespace App\Services;

use App\Models\Payment;
use App\Models\Transaction;
use App\Services\Kafka\KafkaProducer;
use App\Services\PaymentAnalyticsService;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class PaymentService
{
    protected KafkaProducer $kafkaProducer;
    protected FraudDetectionService $fraudService;
    protected PaymentAnalyticsService $analyticsService;

    public function __construct(
        KafkaProducer $kafkaProducer, 
        FraudDetectionService $fraudService,
        PaymentAnalyticsService $analyticsService
    ) {
        $this->kafkaProducer = $kafkaProducer;
        $this->fraudService = $fraudService;
        $this->analyticsService = $analyticsService;
    }

    /**
     * Create and process payment (SYNCHRONOUS)
     * Returns immediate result to user
     */
    public function createPayment(array $data): Payment
    {
        DB::beginTransaction();
        try {
            // Create payment record
            $payment = Payment::create([
                'user_id' => $data['user_id'],
                'amount' => $data['amount'],
                'currency' => $data['currency'] ?? 'VND',
                'payment_method' => $data['payment_method'],
                'merchant_id' => $data['merchant_id'] ?? null,
                'status' => 'PROCESSING',
                'metadata' => $data['metadata'] ?? null,
            ]);

            // Process payment immediately (synchronous)
            $this->processPayment($payment);

            DB::commit();

            // Send to Kafka for logging and notifications (async, fire-and-forget)
            // DISABLED for load testing to avoid message explosion
            // $this->sendPaymentEventToKafka($payment);

            Log::info("Payment processed", [
                'payment_id' => $payment->id,
                'status' => $payment->status
            ]);

            return $payment->fresh();
        } catch (\Exception $e) {
            DB::rollBack();
            Log::error("Payment creation failed: " . $e->getMessage());
            throw $e;
        }
    }

    /**
     * Retry payment and send to Kafka
     */
    public function retryPayment(Payment $payment): bool
    {
        // Process payment
        $result = $this->processPayment($payment);
        
        // Send to Kafka regardless of result
        $this->sendPaymentEventToKafka($payment->fresh());
        
        return $result;
    }

    /**
     * Send payment event to Kafka for logging and notifications
     * This is fire-and-forget, doesn't affect payment result
     */
    protected function sendPaymentEventToKafka(Payment $payment): void
    {
        try {
            $eventData = [
                'id' => $payment->id,
                'user_id' => $payment->user_id,
                'amount' => (float) $payment->amount,
                'currency' => $payment->currency,
                'payment_method' => $payment->payment_method,
                'merchant_id' => $payment->merchant_id,
                'status' => $payment->status,
                'error_code' => $payment->error_code,
                'error_message' => $payment->error_message,
                'processed_at' => $payment->processed_at?->toIso8601String(),
                'timestamp' => now()->toIso8601String(),
            ];

            // Send to main topic for analytics (single source of truth)
            $this->kafkaProducer->send('payment-events', $eventData);

            // Send notifications separately (not for analytics)
            switch ($payment->status) {
                case 'SUCCESS':
                    $this->kafkaProducer->sendNotification([
                        'type' => 'PAYMENT_SUCCESS',
                        'user_id' => $payment->user_id,
                        'payment_id' => $payment->id,
                        'message' => "Payment of {$payment->amount} {$payment->currency} was successful",
                    ]);
                    break;

                case 'FAILED':
                    // TODO: Trigger retry notification if applicable
                    if ($payment->canRetry()) {
                        $this->kafkaProducer->sendNotification([
                            'type' => 'PAYMENT_RETRY',
                            'user_id' => $payment->user_id,
                            'payment_id' => $payment->id,
                            'message' => "Payment failed, retry available",
                        ]);
                    }
                    break;

                case 'FRAUD_DETECTED':
                    $this->kafkaProducer->sendNotification([
                        'type' => 'FRAUD_ALERT',
                        'user_id' => $payment->user_id,
                        'payment_id' => $payment->id,
                        'message' => "Fraud detected on payment",
                    ]);
                    break;
            }

        } catch (\Exception $e) {
            // Don't fail payment if Kafka logging fails
            Log::warning("Failed to send payment event to Kafka: " . $e->getMessage(), [
                'payment_id' => $payment->id
            ]);
        }
    }

    /**
     * Process payment (simulating random success/failure)
     */
    public function processPayment(Payment $payment): bool
    {
        try {
            $payment->update(['status' => 'PROCESSING']);

            // Check for fraud
            if ($this->fraudService->isSuspicious($payment)) {
                return $this->handleFraudDetection($payment);
            }

            // Mock payment gateway for testing (10ms instead of 100-500ms)
            if (env('MOCK_PAYMENT_GATEWAY', false)) {
                usleep(10000); // 10ms - Fast for testing
                $isSuccess = rand(1, 100) <= 70;
            } else {
                // Simulate real payment gateway delay
                usleep(rand(100000, 500000)); // 100-500ms - Realistic
                $isSuccess = rand(1, 100) <= 70;
            }

            // Random success/failure (70% success rate)
            if ($isSuccess) {
                return $this->handleSuccess($payment);
            } else {
                return $this->handleFailure($payment);
            }
        } catch (\Exception $e) {
            Log::error("Payment processing error: " . $e->getMessage(), [
                'payment_id' => $payment->id
            ]);
            return false;
        }
    }

    /**
     * Handle successful payment
     */
    protected function handleSuccess(Payment $payment): bool
    {
        try {
            $payment->update([
                'status' => 'SUCCESS',
                'processed_at' => now(),
            ]);

            // Send email notification via Kafka
            $this->kafkaProducer->send('send-email', [
                'type' => 'PAYMENT_SUCCESS',
                'user_id' => $payment->user_id,
                'payment_id' => $payment->id,
                'email' => $payment->user->email ?? 'user@example.com',
                'subject' => 'Payment Successful',
                'template' => 'payment-success',
                'data' => [
                    'amount' => $payment->amount,
                    'currency' => $payment->currency,
                    'payment_method' => $payment->payment_method,
                    'transaction_id' => $payment->id,
                ],
            ], "user:{$payment->user_id}");
            
            // Send push notification via Kafka
            $this->kafkaProducer->send('send-notification', [
                'type' => 'PAYMENT_SUCCESS',
                'user_id' => $payment->user_id,
                'payment_id' => $payment->id,
                'title' => 'Payment Successful',
                'body' => "Your payment of {$payment->amount} {$payment->currency} was successful",
                'data' => [
                    'payment_id' => $payment->id,
                    'amount' => $payment->amount,
                    'status' => 'SUCCESS',
                ],
            ], "user:{$payment->user_id}");

            Log::info("Payment successful", ['payment_id' => $payment->id]);
            return true;
        } catch (\Exception $e) {
            Log::error("Error handling payment success: " . $e->getMessage());
            return false;
        }
    }

    /**
     * Handle failed payment
     */
    protected function handleFailure(Payment $payment): bool
    {
        try {
            $errorCodes = ['INSUFFICIENT_FUNDS', 'CARD_DECLINED', 'NETWORK_ERROR', 'TIMEOUT', 'INVALID_CARD'];
            $errorCode = $errorCodes[array_rand($errorCodes)];

            $payment->update([
                'status' => 'FAILED',
                'error_code' => $errorCode,
                'error_message' => "Payment failed: {$errorCode}",
                'processed_at' => now(),
            ]);

            // Send email notification via Kafka
            $this->kafkaProducer->send('send-email', [
                'type' => 'PAYMENT_FAILED',
                'user_id' => $payment->user_id,
                'payment_id' => $payment->id,
                'email' => $payment->user->email ?? 'user@example.com',
                'subject' => 'Payment Failed',
                'template' => 'payment-failed',
                'data' => [
                    'amount' => $payment->amount,
                    'currency' => $payment->currency,
                    'error_code' => $errorCode,
                    'error_message' => "Payment failed: {$errorCode}",
                    'can_retry' => $payment->canRetry(),
                ],
            ], "user:{$payment->user_id}");
            
            // Send push notification via Kafka
            $this->kafkaProducer->send('send-notification', [
                'type' => 'PAYMENT_FAILED',
                'user_id' => $payment->user_id,
                'payment_id' => $payment->id,
                'title' => 'Payment Failed',
                'body' => "Your payment of {$payment->amount} {$payment->currency} failed: {$errorCode}",
                'data' => [
                    'payment_id' => $payment->id,
                    'amount' => $payment->amount,
                    'status' => 'FAILED',
                    'error_code' => $errorCode,
                ],
            ], "user:{$payment->user_id}");

            // Increment retry count if can retry
            if ($payment->canRetry()) {
                $payment->incrementRetry();
                Log::warning("Payment failed, can retry", [
                    'payment_id' => $payment->id,
                    'retry_count' => $payment->retry_count,
                    'error_code' => $errorCode,
                ]);
            } else {
                Log::error("Payment permanently failed", [
                    'payment_id' => $payment->id,
                    'error_code' => $errorCode,
                ]);
            }

            return false;
        } catch (\Exception $e) {
            Log::error("Error handling payment failure: " . $e->getMessage());
            return false;
        }
    }

    /**
     * Handle fraud detection
     */
    protected function handleFraudDetection(Payment $payment): bool
    {
        try {
            $payment->update([
                'status' => 'FRAUD_DETECTED',
                'error_code' => 'FRAUD_SUSPECTED',
                'error_message' => 'Transaction flagged for fraud review',
                'processed_at' => now(),
            ]);

            Log::warning("Fraud detected", ['payment_id' => $payment->id]);
            return false;
        } catch (\Exception $e) {
            Log::error("Error handling fraud detection: " . $e->getMessage());
            return false;
        }
    }
}
