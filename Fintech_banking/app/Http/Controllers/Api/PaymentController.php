<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Payment;
use App\Services\PaymentService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Validator;

class PaymentController extends Controller
{
    protected PaymentService $paymentService;

    public function __construct(PaymentService $paymentService)
    {
        $this->paymentService = $paymentService;
    }

    /**
     * Create new payment
     */
    public function store(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'user_id' => 'required|integer|min:1',
            'amount' => 'required|numeric|min:1',
            'currency' => 'nullable|string|size:3',
            'payment_method' => 'required|in:CARD,BANK_TRANSFER,EWALLET,CASH',
            'merchant_id' => 'nullable|string|max:100',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'success' => false,
                'errors' => $validator->errors(),
            ], 422);
        }

        try {
            $payment = $this->paymentService->createPayment($request->all());

            return response()->json([
                'success' => true,
                'message' => 'Payment created successfully',
                'data' => $payment,
            ], 201);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to create payment',
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Get payment by ID
     */
    public function show(int $id): JsonResponse
    {
        $payment = Payment::with(['user', 'transaction'])->find($id);

        if (!$payment) {
            return response()->json([
                'success' => false,
                'message' => 'Payment not found',
            ], 404);
        }

        return response()->json([
            'success' => true,
            'data' => $payment,
        ]);
    }

    /**
     * Get all payments
     */
    public function index(Request $request): JsonResponse
    {
        $query = Payment::with(['user'])->orderBy('created_at', 'desc');

        // Filter by status
        if ($request->has('status')) {
            $query->where('status', $request->status);
        }

        // Filter by user
        if ($request->has('user_id')) {
            $query->where('user_id', $request->user_id);
        }

        $payments = $query->paginate($request->get('per_page', 15));

        return response()->json([
            'success' => true,
            'data' => $payments,
        ]);
    }

    /**
     * Retry failed payment
     */
    public function retry(int $id): JsonResponse
    {
        $payment = Payment::find($id);

        if (!$payment) {
            return response()->json([
                'success' => false,
                'message' => 'Payment not found',
            ], 404);
        }

        if (!$payment->canRetry()) {
            return response()->json([
                'success' => false,
                'message' => 'Payment cannot be retried',
            ], 400);
        }

        try {
            // Retry payment (will process and send to Kafka)
            $this->paymentService->retryPayment($payment);

            return response()->json([
                'success' => true,
                'message' => 'Payment retry completed',
                'data' => $payment->fresh(),
            ]);
        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'message' => 'Failed to retry payment',
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Get payment statistics
     */
    public function statistics(): JsonResponse
    {
        $stats = [
            'total' => Payment::count(),
            'success' => Payment::where('status', 'SUCCESS')->count(),
            'failed' => Payment::where('status', 'FAILED')->count(),
            'pending' => Payment::where('status', 'PENDING')->count(),
            'fraud_detected' => Payment::where('status', 'FRAUD_DETECTED')->count(),
            'total_amount' => Payment::where('status', 'SUCCESS')->sum('amount'),
            'success_rate' => 0,
        ];

        if ($stats['total'] > 0) {
            $stats['success_rate'] = round(($stats['success'] / $stats['total']) * 100, 2);
        }

        return response()->json([
            'success' => true,
            'data' => $stats,
        ]);
    }
}
