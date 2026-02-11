<?php

namespace App\Services;

use Illuminate\Support\Facades\Log;

class EmailService
{
    /**
     * Send email with 99% success rate
     * Simulates OPTIMIZED transactional email service
     * 
     * @param array $data Email data
     * @return bool Success or failure
     */
    public function sendEmail(array $data): bool
    {
        $type = $data['type'] ?? 'unknown';
        $recipient = $data['recipient'] ?? 'unknown';
        
        // Check for optimized mode (100ms flat)
        if (env('OPTIMIZED_EMAIL_SERVICE', false)) {
            // OPTIMIZED: 100ms flat (best case scenario)
            // This represents highly optimized email service with:
            // - Connection pooling
            // - Keep-alive connections
            // - Minimal validation
            // - Fast API endpoint
            usleep(100000); // 100ms
        } else {
            // REALISTIC: 50-200ms (average case)
            // This is realistic for SendGrid, AWS SES, Mailgun, Postmark
            usleep(rand(50000, 200000));
        }
        
        // 99% success rate (realistic for production email services)
        // Only 1% fail rate to avoid retry storm
        $success = rand(1, 100) <= 99;
        
        if ($success) {
            // Minimal logging for performance
            if (rand(1, 1000) <= 1) { // Only log 0.1% of emails
                Log::info("✅ Email sent successfully", [
                    'type' => $type,
                    'recipient' => $recipient,
                ]);
            }
        } else {
            Log::warning("❌ Email sending failed", [
                'type' => $type,
                'recipient' => $recipient,
            ]);
        }
        
        return $success;
    }
    
    /**
     * Send multiple emails concurrently (simulated)
     * 
     * In production, this would use:
     * - Guzzle concurrent requests
     * - ReactPHP promises
     * - Swoole coroutines
     * - Worker pool pattern
     * 
     * @param array $emails Array of email data
     * @return array Results [success_count, failed_count]
     */
    public function sendEmailsConcurrently(array $emails): array
    {
        $startTime = microtime(true);
        $successCount = 0;
        $failedCount = 0;
        
        // Simulate concurrent sending
        // In real async: all emails would be sent "at the same time"
        // So we only wait for the longest one, not sum of all
        
        $maxDelay = 0;
        foreach ($emails as $emailData) {
            $delay = rand(50000, 200000); // 50-200ms
            $maxDelay = max($maxDelay, $delay);
            
            // Simulate send
            $success = rand(1, 100) <= 99; // 99% success rate
            if ($success) {
                $successCount++;
            } else {
                $failedCount++;
            }
        }
        
        // In concurrent mode, we only wait for the longest request
        usleep($maxDelay);
        
        $elapsed = microtime(true) - $startTime;
        
        Log::info("Concurrent email batch completed", [
            'count' => count($emails),
            'success' => $successCount,
            'failed' => $failedCount,
            'elapsed' => round($elapsed, 3) . 's',
            'throughput' => round(count($emails) / $elapsed, 2) . ' msg/s',
        ]);
        
        return [$successCount, $failedCount];
    }
    
    /**
     * Send email with retry logic
     * First attempt: 50% success
     * Retry: 60% success
     * If both fail, stop
     * 
     * NOTE: This method is NOT used in optimized Kafka consumer
     * Kafka consumer uses single attempt + Kafka-based retry instead
     * 
     * @param array $data Email data
     * @return bool Final result
     */
    public function sendEmailWithRetry(array $data): bool
    {
        $type = $data['type'] ?? 'unknown';
        
        // First attempt (50% success)
        Log::info("📧 Email attempt #1", ['type' => $type]);
        
        if ($this->sendEmailFirstAttempt($data)) {
            return true;
        }
        
        // First attempt failed, wait before retry
        Log::warning("⏳ Waiting 2 seconds before retry...", ['type' => $type]);
        sleep(2);
        
        // Retry (60% success)
        Log::info("📧 Email attempt #2 (RETRY)", ['type' => $type]);
        
        if ($this->sendEmailRetry($data)) {
            Log::info("✅ Email sent successfully on RETRY", ['type' => $type]);
            return true;
        }
        
        // Both attempts failed
        Log::error("❌ Email sending FAILED after retry. Stopping.", ['type' => $type]);
        return false;
    }
    
    /**
     * First attempt with 50% success rate
     * Simulates transactional email service (50-200ms)
     */
    protected function sendEmailFirstAttempt(array $data): bool
    {
        usleep(rand(50000, 200000)); // 50-200ms (realistic)
        return rand(1, 100) <= 50;
    }
    
    /**
     * Retry attempt with 60% success rate
     * Simulates transactional email service (50-200ms)
     */
    protected function sendEmailRetry(array $data): bool
    {
        usleep(rand(50000, 200000)); // 50-200ms (realistic)
        return rand(1, 100) <= 60;
    }
}
