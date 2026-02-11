<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Payment #{{ $payment->id }} - Fintech Banking</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 700px;
            margin: 0 auto;
        }
        .card {
            background: white;
            border-radius: 16px;
            padding: 32px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            margin-bottom: 20px;
        }
        h1 {
            color: #1a202c;
            margin-bottom: 8px;
            font-size: 28px;
        }
        .subtitle {
            color: #718096;
            margin-bottom: 24px;
            font-size: 14px;
        }
        .status-badge {
            display: inline-block;
            padding: 8px 16px;
            border-radius: 20px;
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 24px;
        }
        .status-processing {
            background: #dbeafe;
            color: #1e40af;
        }
        .status-success {
            background: #d1fae5;
            color: #065f46;
        }
        .status-failed {
            background: #fee2e2;
            color: #991b1b;
        }
        .status-fraud_detected {
            background: #fce7f3;
            color: #9f1239;
        }
        .info-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
            margin-bottom: 24px;
        }
        .info-item {
            padding: 16px;
            background: #f7fafc;
            border-radius: 8px;
        }
        .info-label {
            color: #718096;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            margin-bottom: 4px;
        }
        .info-value {
            color: #1a202c;
            font-size: 18px;
            font-weight: 600;
        }
        .btn {
            padding: 12px 24px;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            border: none;
            transition: all 0.2s;
            text-decoration: none;
            display: inline-block;
        }
        .btn-primary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .btn-secondary {
            background: #e2e8f0;
            color: #4a5568;
        }
        .btn:hover {
            transform: translateY(-2px);
        }
        .btn-group {
            display: flex;
            gap: 12px;
            margin-top: 24px;
        }
        .back-link {
            display: inline-block;
            color: white;
            text-decoration: none;
            margin-bottom: 20px;
            font-weight: 500;
        }
        .back-link:hover {
            text-decoration: underline;
        }
        .error-box {
            background: #fee2e2;
            border: 1px solid #fca5a5;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 20px;
        }
        .error-title {
            color: #991b1b;
            font-weight: 600;
            margin-bottom: 4px;
        }
        .error-message {
            color: #dc2626;
            font-size: 14px;
        }
        .success-box {
            background: #d1fae5;
            border: 1px solid #6ee7b7;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 20px;
            text-align: center;
        }
        .success-icon {
            font-size: 48px;
            margin-bottom: 8px;
        }
        .success-message {
            color: #065f46;
            font-size: 18px;
            font-weight: 600;
        }
    </style>
</head>
<body>
    <div class="container">
        <a href="{{ route('payments.index') }}" class="back-link">← Back to Payments</a>
        
        <div class="card">
            <h1>💳 Payment #{{ $payment->id }}</h1>
            <div class="subtitle">Created {{ $payment->created_at->diffForHumans() }}</div>
            
            <div class="status-badge status-{{ strtolower($payment->status) }}">
                {{ $payment->status }}
            </div>

            @if($payment->status === 'SUCCESS')
                <div class="success-box">
                    <div class="success-icon">✅</div>
                    <div class="success-message">Payment Successful!</div>
                </div>
            @endif

            @if($payment->error_message)
                <div class="error-box">
                    <div class="error-title">{{ $payment->error_code }}</div>
                    <div class="error-message">{{ $payment->error_message }}</div>
                </div>
            @endif

            <div class="info-grid">
                <div class="info-item">
                    <div class="info-label">Amount</div>
                    <div class="info-value">{{ number_format($payment->amount, 0) }} {{ $payment->currency }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Payment Method</div>
                    <div class="info-value">{{ $payment->payment_method }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">User</div>
                    <div class="info-value">{{ $payment->user->name }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Merchant</div>
                    <div class="info-value">{{ $payment->merchant_id ?? 'N/A' }}</div>
                </div>
                @if($payment->retry_count > 0)
                <div class="info-item">
                    <div class="info-label">Retry Count</div>
                    <div class="info-value">{{ $payment->retry_count }}/3</div>
                </div>
                @endif
                @if($payment->processed_at)
                <div class="info-item">
                    <div class="info-label">Processed At</div>
                    <div class="info-value">{{ $payment->processed_at->format('Y-m-d H:i:s') }}</div>
                </div>
                @endif
            </div>

            <div class="btn-group">
                <a href="{{ route('payments.create') }}" class="btn btn-primary">New Payment</a>
                <a href="{{ route('payments.index') }}" class="btn btn-secondary">View All</a>
                @if($payment->canRetry())
                <button onclick="retryPayment()" class="btn btn-primary" id="retry-btn">Retry Payment</button>
                @endif
            </div>
        </div>
    </div>

    <script>
        const paymentId = {{ $payment->id }};

        async function retryPayment() {
            const retryBtn = document.getElementById('retry-btn');
            retryBtn.disabled = true;
            retryBtn.textContent = 'Retrying...';

            try {
                const response = await fetch(`/api/payments/${paymentId}/retry`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Accept': 'application/json',
                    }
                });

                const result = await response.json();

                if (result.success) {
                    location.reload();
                } else {
                    alert('Failed to retry payment: ' + result.message);
                    retryBtn.disabled = false;
                    retryBtn.textContent = 'Retry Payment';
                }
            } catch (error) {
                alert('Error: ' + error.message);
                retryBtn.disabled = false;
                retryBtn.textContent = 'Retry Payment';
            }
        }
    </script>
</body>
</html>
