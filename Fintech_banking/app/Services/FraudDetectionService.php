<?php

namespace App\Services;

use App\Models\Payment;
use Illuminate\Support\Facades\Log;

class FraudDetectionService
{
    /**
     * Check if payment is suspicious
     */
    public function isSuspicious(Payment $payment): bool
    {
        // Skip fraud detection if disabled (for load testing)
        if (config('services.fraud_detection.enabled', true) === false) {
            return false;
        }

        // Rule 1: Large transactions (> 50,000) have 10% chance of being flagged
        if ($payment->amount > 50000) {
            if (rand(1, 100) <= 10) {
                Log::warning("Fraud: Large transaction detected", [
                    'payment_id' => $payment->id,
                    'amount' => $payment->amount,
                ]);
                return true;
            }
        }

        // Rule 2: Multiple failed attempts from same user (credential stuffing)
        $recentFailures = Payment::where('user_id', $payment->user_id)
            ->where('status', 'FAILED')
            ->where('created_at', '>=', now()->subHours(1))
            ->count();

        if ($recentFailures >= 3) {
            Log::warning("Fraud: Multiple failed attempts", [
                'payment_id' => $payment->id,
                'user_id' => $payment->user_id,
                'failures' => $recentFailures,
            ]);
            return true;
        }

        // Rule 3: Rapid transactions - Multiple payments in short time
        $last5Minutes = Payment::where('user_id', $payment->user_id)
            ->where('created_at', '>=', now()->subMinutes(5))
            ->count();

        if ($last5Minutes >= 5) {
            Log::warning("Fraud: Rapid transactions (5 min)", [
                'payment_id' => $payment->id,
                'user_id' => $payment->user_id,
                'count' => $last5Minutes,
            ]);
            return true;
        }

        // Rule 4: High frequency - Too many payments in 1 hour
        $lastHour = Payment::where('user_id', $payment->user_id)
            ->where('created_at', '>=', now()->subHour())
            ->count();

        if ($lastHour >= 10) {
            Log::warning("Fraud: High frequency (1 hour)", [
                'payment_id' => $payment->id,
                'user_id' => $payment->user_id,
                'count' => $lastHour,
            ]);
            return true;
        }

        // Rule 5: Duplicate amount - Same amount multiple times in short period
        $duplicateAmount = Payment::where('user_id', $payment->user_id)
            ->where('amount', $payment->amount)
            ->where('created_at', '>=', now()->subMinutes(10))
            ->count();

        if ($duplicateAmount >= 3) {
            Log::warning("Fraud: Duplicate amount detected", [
                'payment_id' => $payment->id,
                'user_id' => $payment->user_id,
                'amount' => $payment->amount,
                'count' => $duplicateAmount,
            ]);
            return true;
        }

        // Rule 6: Velocity check - Total amount in short time
        $totalLast10Min = Payment::where('user_id', $payment->user_id)
            ->where('created_at', '>=', now()->subMinutes(10))
            ->sum('amount');

        if ($totalLast10Min > 100000) { // > 100K in 10 minutes
            Log::warning("Fraud: High velocity spending", [
                'payment_id' => $payment->id,
                'user_id' => $payment->user_id,
                'total_amount' => $totalLast10Min,
            ]);
            return true;
        }

        return false;
    }

    /**
     * Analyze fraud patterns
     */
    public function analyzeFraudPatterns(array $fraudData): array
    {
        // Simulate fraud analysis
        return [
            'risk_score' => rand(70, 100),
            'patterns_detected' => [
                'large_transaction' => $fraudData['amount'] > 50000,
                'suspicious_merchant' => rand(0, 1) === 1,
                'unusual_time' => date('H') < 6 || date('H') > 22,
            ],
            'recommendation' => 'BLOCK',
            'analyzed_at' => now()->toIso8601String(),
        ];
    }
}
