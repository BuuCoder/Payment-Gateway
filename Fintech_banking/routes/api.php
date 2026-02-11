<?php

use App\Http\Controllers\Api\PaymentController;
use App\Http\Controllers\Api\AnalyticsController;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;

Route::get('/user', function (Request $request) {
    return $request->user();
})->middleware('auth:sanctum');

// Payment routes
Route::prefix('payments')->group(function () {
    Route::get('/', [PaymentController::class, 'index']);
    Route::post('/', [PaymentController::class, 'store']);
    Route::get('/statistics', [PaymentController::class, 'statistics']);
    Route::get('/{id}', [PaymentController::class, 'show']);
    Route::post('/{id}/retry', [PaymentController::class, 'retry']);
});

// Analytics routes
Route::prefix('analytics')->group(function () {
    Route::get('/dashboard', [AnalyticsController::class, 'dashboard']);
    Route::get('/revenue-trend', [AnalyticsController::class, 'revenueTrend']);
    Route::get('/top-merchants', [AnalyticsController::class, 'topMerchants']);
    Route::get('/fraud-patterns', [AnalyticsController::class, 'fraudPatterns']);
});

// Health check
Route::get('/health', function () {
    return response()->json([
        'status' => 'ok',
        'timestamp' => now()->toIso8601String(),
        'service' => 'Fintech Banking API',
    ]);
});

