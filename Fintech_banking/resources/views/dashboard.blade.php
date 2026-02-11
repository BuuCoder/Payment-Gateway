<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Dashboard - Fintech Banking</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
            padding: 20px;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        .header {
            background: white;
            padding: 20px 30px;
            border-radius: 12px;
            margin-bottom: 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .header h1 {
            font-size: 28px;
            color: #1a202c;
        }
        .nav-links {
            display: flex;
            gap: 15px;
        }
        .nav-link {
            padding: 10px 20px;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 6px;
            font-weight: 600;
            transition: all 0.2s;
        }
        .nav-link:hover {
            background: #5568d3;
            transform: translateY(-2px);
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 20px;
            margin-bottom: 20px;
        }
        .card {
            background: white;
            padding: 24px;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .card.large {
            grid-column: span 2;
        }
        .card.full {
            grid-column: span 4;
        }
        .card h2 {
            font-size: 14px;
            color: #718096;
            margin-bottom: 8px;
            text-transform: uppercase;
            font-weight: 600;
        }
        .card .value {
            font-size: 32px;
            font-weight: bold;
            color: #1a202c;
        }
        .card .change {
            font-size: 14px;
            margin-top: 8px;
        }
        .change.positive { color: #10b981; }
        .change.negative { color: #ef4444; }
        .table-container {
            overflow-x: auto;
        }
        table {
            width: 100%;
            border-collapse: collapse;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }
        th {
            background: #f7fafc;
            font-weight: 600;
            color: #4a5568;
            font-size: 12px;
            text-transform: uppercase;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 600;
        }
        .status-success { background: #d1fae5; color: #065f46; }
        .status-failed { background: #fee2e2; color: #991b1b; }
        .status-fraud { background: #fce7f3; color: #9f1239; }
        .refresh-indicator {
            display: inline-block;
            width: 8px;
            height: 8px;
            background: #10b981;
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 Performance Dashboard</h1>
            <div class="nav-links">
                <a href="/payments" class="nav-link">💳 Payments</a>
                <a href="/load-test" class="nav-link">🚀 Load Test</a>
                <a href="http://localhost:8081" target="_blank" class="nav-link">📈 Kafka UI</a>
            </div>
        </div>

        <div class="grid">
            <div class="card">
                <h2>Total Payments</h2>
                <div class="value" id="totalPayments">-</div>
                <div class="change positive" id="paymentsChange">
                    <span class="refresh-indicator"></span>Live
                </div>
            </div>
            <div class="card">
                <h2>Success Rate</h2>
                <div class="value" id="successRate">-</div>
                <div class="change" id="successChange">-</div>
            </div>
            <div class="card">
                <h2>Total Revenue</h2>
                <div class="value" id="totalRevenue">-</div>
                <div class="change positive" id="revenueChange">-</div>
            </div>
            <div class="card">
                <h2>Fraud Rate</h2>
                <div class="value" id="fraudRate">-</div>
                <div class="change" id="fraudChange">-</div>
            </div>
        </div>

        <div class="grid">
            <div class="card large">
                <h2>By Status</h2>
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Status</th>
                                <th>Count</th>
                                <th>Total Amount</th>
                            </tr>
                        </thead>
                        <tbody id="statusTable">
                            <tr><td colspan="3">Loading...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <div class="card large">
                <h2>By Payment Method</h2>
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Method</th>
                                <th>Count</th>
                                <th>Total Amount</th>
                            </tr>
                        </thead>
                        <tbody id="methodTable">
                            <tr><td colspan="3">Loading...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <div class="card full">
            <h2>Recent Failures</h2>
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Payment ID</th>
                            <th>User ID</th>
                            <th>Amount</th>
                            <th>Status</th>
                            <th>Error Code</th>
                            <th>Error Message</th>
                            <th>Retry Count</th>
                            <th>Failed At</th>
                        </tr>
                    </thead>
                    <tbody id="failuresTable">
                        <tr><td colspan="8">Loading...</td></tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <script>
        async function fetchDashboard() {
            try {
                const response = await fetch('/api/analytics/dashboard');
                const data = await response.json();
                
                if (data.success && data.data) {
                    updateDashboard(data.data);
                }
            } catch (error) {
                console.error('Failed to fetch dashboard:', error);
            }
        }

        function updateDashboard(data) {
            // Update main metrics
            if (data.today) {
                document.getElementById('totalPayments').textContent = data.today.total_payments;
                document.getElementById('successRate').textContent = data.today.success_rate + '%';
                document.getElementById('totalRevenue').textContent = formatCurrency(data.today.total_revenue);
                document.getElementById('fraudRate').textContent = data.today.fraud_rate + '%';
                
                // Update change indicators
                const successChange = document.getElementById('successChange');
                if (data.today.success_rate >= 95) {
                    successChange.textContent = '✅ Excellent';
                    successChange.className = 'change positive';
                } else if (data.today.success_rate >= 90) {
                    successChange.textContent = '⚠️ Good';
                    successChange.className = 'change';
                } else {
                    successChange.textContent = '❌ Poor';
                    successChange.className = 'change negative';
                }
                
                const fraudChange = document.getElementById('fraudChange');
                if (data.today.fraud_rate === 0) {
                    fraudChange.textContent = '✅ None';
                    fraudChange.className = 'change positive';
                } else if (data.today.fraud_rate < 5) {
                    fraudChange.textContent = '⚠️ Low';
                    fraudChange.className = 'change';
                } else {
                    fraudChange.textContent = '❌ High';
                    fraudChange.className = 'change negative';
                }
            }
            
            // Update status table
            if (data.by_status) {
                const statusTable = document.getElementById('statusTable');
                statusTable.innerHTML = data.by_status.map(item => `
                    <tr>
                        <td><span class="status-badge status-${item.status.toLowerCase()}">${item.status}</span></td>
                        <td>${item.count}</td>
                        <td>${formatCurrency(item.total)}</td>
                    </tr>
                `).join('');
            }
            
            // Update method table
            if (data.by_method) {
                const methodTable = document.getElementById('methodTable');
                methodTable.innerHTML = data.by_method.map(item => `
                    <tr>
                        <td>${item.payment_method}</td>
                        <td>${item.count}</td>
                        <td>${formatCurrency(item.total)}</td>
                    </tr>
                `).join('');
            }
            
            // Update failures table
            if (data.recent_failures && data.recent_failures.length > 0) {
                const failuresTable = document.getElementById('failuresTable');
                failuresTable.innerHTML = data.recent_failures.map(item => `
                    <tr>
                        <td>#${item.payment_id}</td>
                        <td>${item.user_id}</td>
                        <td>${formatCurrency(item.amount)} ${item.currency}</td>
                        <td><span class="status-badge status-${item.status.toLowerCase()}">${item.status}</span></td>
                        <td>${item.error_code || '-'}</td>
                        <td>${item.error_message || '-'}</td>
                        <td>${item.retry_count}/3</td>
                        <td>${formatDate(item.failed_at)}</td>
                    </tr>
                `).join('');
            } else {
                document.getElementById('failuresTable').innerHTML = '<tr><td colspan="8" style="text-align: center; color: #10b981;">✅ No recent failures</td></tr>';
            }
        }

        function formatCurrency(amount) {
            return new Intl.NumberFormat().format(amount);
        }

        function formatDate(dateString) {
            const date = new Date(dateString);
            return date.toLocaleString();
        }

        // Fetch on load and every 3 seconds
        fetchDashboard();
        setInterval(fetchDashboard, 3000);
    </script>
</body>
</html>
