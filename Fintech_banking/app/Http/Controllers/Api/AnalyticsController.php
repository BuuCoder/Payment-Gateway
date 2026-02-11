<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Services\PaymentAnalyticsService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class AnalyticsController extends Controller
{
    protected PaymentAnalyticsService $analyticsService;

    public function __construct(PaymentAnalyticsService $analyticsService)
    {
        $this->analyticsService = $analyticsService;
    }

    /**
     * Get dashboard statistics
     */
    public function dashboard(): JsonResponse
    {
        try {
            $stats = $this->analyticsService->getDashboardStats();

            return response()->json([
                'success' => true,
                'data' => $stats,
            ]);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to fetch dashboard stats',
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Get revenue trend
     */
    public function revenueTrend(Request $request): JsonResponse
    {
        try {
            $days = $request->get('days', 7);
            $trend = $this->analyticsService->getRevenueTrend($days);

            return response()->json([
                'success' => true,
                'data' => $trend,
            ]);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to fetch revenue trend',
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Get top merchants
     */
    public function topMerchants(Request $request): JsonResponse
    {
        try {
            $limit = $request->get('limit', 10);
            $merchants = $this->analyticsService->getTopMerchants($limit);

            return response()->json([
                'success' => true,
                'data' => $merchants,
            ]);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to fetch top merchants',
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Get fraud patterns
     */
    public function fraudPatterns(): JsonResponse
    {
        try {
            $patterns = $this->analyticsService->getFraudPatterns();

            return response()->json([
                'success' => true,
                'data' => $patterns,
            ]);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to fetch fraud patterns',
                'error' => $e->getMessage(),
            ], 500);
        }
    }
}
