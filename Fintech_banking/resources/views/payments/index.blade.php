<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Payments - Fintech Banking</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 24px;
        }
        h1 {
            color: white;
            font-size: 32px;
        }
        .btn {
            padding: 12px 24px;
            background: white;
            color: #667eea;
            border: none;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            transition: transform 0.2s;
        }
        .btn:hover {
            transform: translateY(-2px);
        }
        .card {
            background: white;
            border-radius: 16px;
            padding: 24px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow-x: auto;
        }
        table {
            width: 100%;
            border-collapse: collapse;
        }
        th {
            text-align: left;
            padding: 12px;
            background: #f7fafc;
            color: #4a5568;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            border-bottom: 2px solid #e2e8f0;
        }
        td {
            padding: 16px 12px;
            border-bottom: 1px solid #e2e8f0;
            color: #2d3748;
        }
        tr:hover {
            background: #f7fafc;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 600;
        }
        .status-pending { background: #fef3c7; color: #92400e; }
        .status-processing { background: #dbeafe; color: #1e40af; }
        .status-success { background: #d1fae5; color: #065f46; }
        .status-failed { background: #fee2e2; color: #991b1b; }
        .status-fraud_detected { background: #fce7f3; color: #9f1239; }
        .view-link {
            color: #667eea;
            text-decoration: none;
            font-weight: 500;
        }
        .view-link:hover {
            text-decoration: underline;
        }
        .pagination {
            margin-top: 20px;
            display: flex;
            justify-content: center;
            gap: 8px;
        }
        .pagination a, .pagination span {
            padding: 8px 12px;
            background: white;
            color: #667eea;
            text-decoration: none;
            border-radius: 6px;
            font-size: 14px;
        }
        .pagination .active {
            background: #667eea;
            color: white;
        }
        .empty-state {
            text-align: center;
            padding: 60px 20px;
            color: #718096;
        }
        .empty-state-icon {
            font-size: 64px;
            margin-bottom: 16px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>💳 Payments</h1>
            <a href="{{ route('payments.create') }}" class="btn">+ New Payment</a>
        </div>

        <div class="card">
            @if($payments->count() > 0)
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>User</th>
                            <th>Amount</th>
                            <th>Method</th>
                            <th>Status</th>
                            <th>Created</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        @foreach($payments as $payment)
                        <tr>
                            <td>#{{ $payment->id }}</td>
                            <td>{{ $payment->user->name }}</td>
                            <td>{{ number_format($payment->amount, 0) }} {{ $payment->currency }}</td>
                            <td>{{ $payment->payment_method }}</td>
                            <td>
                                <span class="status-badge status-{{ strtolower($payment->status) }}">
                                    {{ $payment->status }}
                                </span>
                            </td>
                            <td>{{ $payment->created_at->diffForHumans() }}</td>
                            <td>
                                <a href="{{ route('payments.show', $payment->id) }}" class="view-link">
                                    View →
                                </a>
                            </td>
                        </tr>
                        @endforeach
                    </tbody>
                </table>

                <div class="pagination">
                    {{ $payments->links('pagination::simple-default') }}
                </div>
            @else
                <div class="empty-state">
                    <div class="empty-state-icon">💳</div>
                    <h2>No payments yet</h2>
                    <p>Create your first payment to get started</p>
                </div>
            @endif
        </div>
    </div>
</body>
</html>
