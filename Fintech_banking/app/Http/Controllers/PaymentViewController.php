<?php

namespace App\Http\Controllers;

use App\Models\Payment;
use Illuminate\Http\Request;

class PaymentViewController extends Controller
{
    public function index()
    {
        $payments = Payment::with('user')
            ->orderBy('created_at', 'desc')
            ->paginate(10);
        
        return view('payments.index', compact('payments'));
    }

    public function create()
    {
        return view('payments.create');
    }

    public function show($id)
    {
        $payment = Payment::with('user')->findOrFail($id);
        return view('payments.show', compact('payment'));
    }

    /**
     * Load testing dashboard
     */
    public function loadTest()
    {
        return view('load-test');
    }

    /**
     * Performance dashboard
     */
    public function dashboard()
    {
        return view('dashboard');
    }

    /**
     * SSE endpoint for realtime payment status updates
     */
    public function stream($id)
    {
        header('Content-Type: text/event-stream');
        header('Cache-Control: no-cache');
        header('Connection: keep-alive');
        header('X-Accel-Buffering: no'); // Disable nginx buffering

        $payment = Payment::find($id);
        
        if (!$payment) {
            echo "data: " . json_encode(['error' => 'Payment not found']) . "\n\n";
            flush();
            return;
        }

        $lastStatus = $payment->status;
        $maxAttempts = 60; // 60 seconds timeout
        $attempt = 0;

        while ($attempt < $maxAttempts) {
            $payment->refresh();
            
            // Send update if status changed or every 5 seconds
            if ($payment->status !== $lastStatus || $attempt % 5 === 0) {
                $data = [
                    'id' => $payment->id,
                    'status' => $payment->status,
                    'amount' => $payment->amount,
                    'currency' => $payment->currency,
                    'error_code' => $payment->error_code,
                    'error_message' => $payment->error_message,
                    'retry_count' => $payment->retry_count,
                    'processed_at' => $payment->processed_at?->toIso8601String(),
                ];

                echo "data: " . json_encode($data) . "\n\n";
                flush();

                $lastStatus = $payment->status;

                // Close connection if payment is in final state
                if (in_array($payment->status, ['SUCCESS', 'FAILED', 'FRAUD_DETECTED'])) {
                    break;
                }
            }

            sleep(1);
            $attempt++;
        }

        // Send timeout message
        if ($attempt >= $maxAttempts) {
            echo "data: " . json_encode(['status' => 'TIMEOUT']) . "\n\n";
            flush();
        }
    }
}
