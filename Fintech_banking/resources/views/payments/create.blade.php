<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>Create Payment - Fintech Banking</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 600px;
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
            margin-bottom: 24px;
            font-size: 28px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            color: #4a5568;
            font-weight: 600;
            margin-bottom: 8px;
            font-size: 14px;
        }
        input, select {
            width: 100%;
            padding: 12px 16px;
            border: 2px solid #e2e8f0;
            border-radius: 8px;
            font-size: 16px;
            transition: all 0.3s;
        }
        input:focus, select:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
        }
        .btn {
            width: 100%;
            padding: 14px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s;
        }
        .btn:hover {
            transform: translateY(-2px);
        }
        .btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
            transform: none;
        }
        .alert {
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 20px;
            font-size: 14px;
        }
        .alert-error {
            background: #fed7d7;
            color: #c53030;
            border: 1px solid #fc8181;
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
    </style>
</head>
<body>
    <div class="container">
        <a href="{{ route('payments.index') }}" class="back-link">← Back to Payments</a>
        
        <div class="card">
            <h1>💳 Create Payment</h1>
            
            <div id="error-message" class="alert alert-error" style="display: none;"></div>
            
            <form id="payment-form">
                <div class="form-group">
                    <label>User ID</label>
                    <input type="number" name="user_id" value="1" required>
                </div>

                <div class="form-group">
                    <label>Amount</label>
                    <input type="number" name="amount" placeholder="50000" required min="1">
                </div>

                <div class="form-group">
                    <label>Currency</label>
                    <select name="currency">
                        <option value="VND">VND</option>
                        <option value="USD">USD</option>
                        <option value="EUR">EUR</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>Payment Method</label>
                    <select name="payment_method" required>
                        <option value="CARD">Credit/Debit Card</option>
                        <option value="BANK_TRANSFER">Bank Transfer</option>
                        <option value="EWALLET">E-Wallet</option>
                        <option value="CASH">Cash</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>Merchant ID (Optional)</label>
                    <input type="text" name="merchant_id" placeholder="MERCHANT_001">
                </div>

                <button type="submit" class="btn" id="submit-btn">
                    Create Payment
                </button>
            </form>
        </div>
    </div>

    <script>
        const form = document.getElementById('payment-form');
        const submitBtn = document.getElementById('submit-btn');
        const errorDiv = document.getElementById('error-message');

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            
            submitBtn.disabled = true;
            submitBtn.textContent = 'Creating...';
            errorDiv.style.display = 'none';

            const formData = new FormData(form);
            const data = Object.fromEntries(formData.entries());

            try {
                const response = await fetch('/api/payments', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Accept': 'application/json',
                    },
                    body: JSON.stringify(data)
                });

                const result = await response.json();

                if (result.success) {
                    // Redirect to payment status page
                    window.location.href = `/payments/${result.data.id}`;
                } else {
                    throw new Error(result.message || 'Payment creation failed');
                }
            } catch (error) {
                errorDiv.textContent = error.message;
                errorDiv.style.display = 'block';
                submitBtn.disabled = false;
                submitBtn.textContent = 'Create Payment';
            }
        });
    </script>
</body>
</html>
