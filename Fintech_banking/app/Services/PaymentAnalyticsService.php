<?php

namespace App\Services;

use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class PaymentAnalyticsService
{
    /**
     * Process Kafka message and store in analytics table
     * This transforms JSON to structured data for fast queries
     */
    public function processPaymentEvent(array $eventData): void
    {
        try {
            // Convert ISO8601 timestamps to MySQL format
            $processedAt = isset($eventData['processed_at']) 
                ? date('Y-m-d H:i:s', strtotime($eventData['processed_at']))
                : null;
            
            $eventTimestamp = isset($eventData['timestamp'])
                ? date('Y-m-d H:i:s', strtotime($eventData['timestamp']))
                : now();

            // Check if this exact event already exists (prevent duplicate Kafka messages)
            $exists = DB::table('payment_analytics')
                ->where('payment_id', $eventData['id'])
                ->where('status', $eventData['status'])
                ->where('event_timestamp', $eventTimestamp)
                ->exists();

            if ($exists) {
                return; // Skip duplicate
            }

            // Check for previous event BEFORE inserting new one
            $previousEvent = DB::table('payment_analytics')
                ->where('payment_id', $eventData['id'])
                ->orderBy('id', 'desc')
                ->first();

            // Insert new analytics event
            DB::table('payment_analytics')->insert([
                'payment_id' => $eventData['id'],
                'user_id' => $eventData['user_id'],
                'amount' => $eventData['amount'],
                'currency' => $eventData['currency'],
                'status' => $eventData['status'],
                'payment_method' => $eventData['payment_method'],
                'merchant_id' => $eventData['merchant_id'] ?? null,
                'error_code' => $eventData['error_code'] ?? null,
                'error_message' => $eventData['error_message'] ?? null,
                'retry_count' => $eventData['retry_count'] ?? 0,
                'processed_at' => $processedAt,
                'event_timestamp' => $eventTimestamp,
                'created_at' => now(),
                'updated_at' => now(),
            ]);

            // Update aggregations with previous event info
            $this->updateHourlyStats($eventData, $previousEvent);
            $this->updateDailyStats($eventData, $previousEvent);

        } catch (\Exception $e) {
            Log::error("Analytics processing error: " . $e->getMessage(), [
                'event' => $eventData
            ]);
        }
    }

    /**
     * Update hourly statistics
     * Recalculate from latest payment statuses
     */
    protected function updateHourlyStats(array $eventData, $previousEvent = null): void
    {
        $timestamp = $eventData['timestamp'] ?? now();
        $date = date('Y-m-d', strtotime($timestamp));
        $hour = date('H', strtotime($timestamp));

        // Get all unique payments in this hour with their latest status
        $latestPayments = DB::select("
            SELECT pa1.*
            FROM payment_analytics pa1
            INNER JOIN (
                SELECT payment_id, MAX(id) as max_id
                FROM payment_analytics
                WHERE DATE(created_at) = ? AND HOUR(created_at) = ?
                GROUP BY payment_id
            ) pa2 ON pa1.id = pa2.max_id
        ", [$date, $hour]);

        // Group by currency and status
        $stats = collect($latestPayments)->groupBy(function($item) {
            return $item->currency . '|' . $item->status;
        });

        // Clear existing stats for this hour
        DB::table('payment_hourly_stats')
            ->where('date', $date)
            ->where('hour', $hour)
            ->delete();

        // Insert recalculated stats
        foreach ($stats as $key => $group) {
            [$currency, $status] = explode('|', $key);
            $count = $group->count();
            $total = $group->sum('amount');

            DB::table('payment_hourly_stats')->insert([
                'date' => $date,
                'hour' => $hour,
                'currency' => $currency,
                'status' => $status,
                'count' => $count,
                'total_amount' => $total,
                'avg_amount' => $total / $count,
                'created_at' => now(),
                'updated_at' => now(),
            ]);
        }
    }

    /**
     * Update daily statistics
     * Recalculate from latest payment statuses
     */
    protected function updateDailyStats(array $eventData, $previousEvent = null): void
    {
        $timestamp = $eventData['timestamp'] ?? now();
        $date = date('Y-m-d', strtotime($timestamp));

        // Get all unique payments in this day with their latest status
        $latestPayments = DB::select("
            SELECT pa1.*
            FROM payment_analytics pa1
            INNER JOIN (
                SELECT payment_id, MAX(id) as max_id
                FROM payment_analytics
                WHERE DATE(created_at) = ?
                GROUP BY payment_id
            ) pa2 ON pa1.id = pa2.max_id
        ", [$date]);

        // Group by currency and status
        $stats = collect($latestPayments)->groupBy(function($item) {
            return $item->currency . '|' . $item->status;
        });

        // Clear existing stats for this day
        DB::table('payment_daily_stats')
            ->where('date', $date)
            ->delete();

        // Insert recalculated stats
        foreach ($stats as $key => $group) {
            [$currency, $status] = explode('|', $key);
            $count = $group->count();
            $total = $group->sum('amount');
            $amounts = $group->pluck('amount');

            DB::table('payment_daily_stats')->insert([
                'date' => $date,
                'currency' => $currency,
                'status' => $status,
                'count' => $count,
                'total_amount' => $total,
                'avg_amount' => $total / $count,
                'min_amount' => $amounts->min(),
                'max_amount' => $amounts->max(),
                'created_at' => now(),
                'updated_at' => now(),
            ]);
        }
    }

    /**
     * Get real-time dashboard statistics
     * Uses latest status per payment (not all events)
     */
    public function getDashboardStats(): array
    {
        $today = date('Y-m-d');

        // Get latest status for each payment
        $latestStatuses = DB::table('payment_analytics as pa1')
            ->whereDate('pa1.created_at', $today)
            ->whereRaw('pa1.id = (
                SELECT MAX(pa2.id) 
                FROM payment_analytics pa2 
                WHERE pa2.payment_id = pa1.payment_id
            )')
            ->get();

        $totalPayments = $latestStatuses->count();
        $successPayments = $latestStatuses->where('status', 'SUCCESS')->count();
        $fraudPayments = $latestStatuses->where('status', 'FRAUD_DETECTED')->count();

        return [
            'today' => [
                'total_payments' => $totalPayments,
                'total_revenue' => $latestStatuses->where('status', 'SUCCESS')->sum('amount'),
                'success_rate' => $totalPayments > 0 ? round(($successPayments / $totalPayments) * 100, 2) : 0,
                'fraud_rate' => $totalPayments > 0 ? round(($fraudPayments / $totalPayments) * 100, 2) : 0,
            ],
            'by_status' => $latestStatuses->groupBy('status')->map(function ($group, $status) {
                return [
                    'status' => $status,
                    'count' => $group->count(),
                    'total' => $group->sum('amount'),
                ];
            })->values(),
            'by_method' => $latestStatuses->groupBy('payment_method')->map(function ($group, $method) {
                return [
                    'payment_method' => $method,
                    'count' => $group->count(),
                    'total' => $group->sum('amount'),
                ];
            })->values(),
            'recent_failures' => $this->getRecentFailures(),
        ];
    }

    /**
     * Get recent failed or problematic payments
     */
    protected function getRecentFailures(int $limit = 10): array
    {
        // Get latest failed/fraud payments with details
        $failures = DB::table('payment_analytics as pa1')
            ->whereIn('pa1.status', ['FAILED', 'FRAUD_DETECTED'])
            ->whereRaw('pa1.id = (
                SELECT MAX(pa2.id) 
                FROM payment_analytics pa2 
                WHERE pa2.payment_id = pa1.payment_id
            )')
            ->orderBy('pa1.created_at', 'desc')
            ->limit($limit)
            ->get([
                'pa1.payment_id',
                'pa1.user_id',
                'pa1.amount',
                'pa1.currency',
                'pa1.status',
                'pa1.payment_method',
                'pa1.error_code',
                'pa1.error_message',
                'pa1.retry_count',
                'pa1.created_at'
            ]);

        return $failures->map(function($failure) {
            return [
                'payment_id' => $failure->payment_id,
                'user_id' => $failure->user_id,
                'amount' => $failure->amount,
                'currency' => $failure->currency,
                'status' => $failure->status,
                'payment_method' => $failure->payment_method,
                'error_code' => $failure->error_code,
                'error_message' => $failure->error_message,
                'retry_count' => $failure->retry_count,
                'failed_at' => $failure->created_at,
            ];
        })->toArray();
    }

    /**
     * Get success rate for a date
     */
    protected function getSuccessRate(string $date): float
    {
        $total = DB::table('payment_analytics')
            ->whereDate('created_at', $date)
            ->count();

        if ($total === 0) {
            return 0;
        }

        $success = DB::table('payment_analytics')
            ->whereDate('created_at', $date)
            ->where('status', 'SUCCESS')
            ->count();

        return round(($success / $total) * 100, 2);
    }

    /**
     * Get fraud rate for a date
     */
    protected function getFraudRate(string $date): float
    {
        $total = DB::table('payment_analytics')
            ->whereDate('created_at', $date)
            ->count();

        if ($total === 0) {
            return 0;
        }

        $fraud = DB::table('payment_analytics')
            ->whereDate('created_at', $date)
            ->where('status', 'FRAUD_DETECTED')
            ->count();

        return round(($fraud / $total) * 100, 2);
    }

    /**
     * Get revenue trend (last 7 days)
     */
    public function getRevenueTrend(int $days = 7): array
    {
        return DB::table('payment_daily_stats')
            ->where('date', '>=', now()->subDays($days)->format('Y-m-d'))
            ->where('status', 'SUCCESS')
            ->select('date', 'currency', DB::raw('SUM(total_amount) as revenue'))
            ->groupBy('date', 'currency')
            ->orderBy('date')
            ->get()
            ->toArray();
    }

    /**
     * Get top merchants by revenue
     */
    public function getTopMerchants(int $limit = 10): array
    {
        return DB::table('payment_analytics')
            ->where('status', 'SUCCESS')
            ->whereNotNull('merchant_id')
            ->select(
                'merchant_id',
                DB::raw('COUNT(*) as transaction_count'),
                DB::raw('SUM(amount) as total_revenue'),
                DB::raw('AVG(amount) as avg_amount')
            )
            ->groupBy('merchant_id')
            ->orderByDesc('total_revenue')
            ->limit($limit)
            ->get()
            ->toArray();
    }

    /**
     * Get fraud patterns
     */
    public function getFraudPatterns(): array
    {
        return [
            'by_amount_range' => DB::table('payment_analytics')
                ->where('status', 'FRAUD_DETECTED')
                ->select(
                    DB::raw('CASE 
                        WHEN amount < 50000 THEN "< 50K"
                        WHEN amount < 100000 THEN "50K-100K"
                        WHEN amount < 500000 THEN "100K-500K"
                        ELSE "> 500K"
                    END as amount_range'),
                    DB::raw('COUNT(*) as count')
                )
                ->groupBy('amount_range')
                ->get(),
            'by_payment_method' => DB::table('payment_analytics')
                ->where('status', 'FRAUD_DETECTED')
                ->select('payment_method', DB::raw('COUNT(*) as count'))
                ->groupBy('payment_method')
                ->get(),
            'by_hour' => DB::table('payment_analytics')
                ->where('status', 'FRAUD_DETECTED')
                ->select(DB::raw('HOUR(created_at) as hour'), DB::raw('COUNT(*) as count'))
                ->groupBy('hour')
                ->orderBy('hour')
                ->get(),
        ];
    }
}
