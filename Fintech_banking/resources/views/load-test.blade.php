<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>Load Testing Dashboard - Fintech Banking</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        .header {
            text-align: center;
            color: white;
            margin-bottom: 30px;
        }
        .header h1 {
            font-size: 36px;
            margin-bottom: 10px;
        }
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 20px;
        }
        .card {
            background: white;
            border-radius: 16px;
            padding: 24px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        .card h2 {
            color: #1a202c;
            margin-bottom: 20px;
            font-size: 20px;
            border-bottom: 2px solid #667eea;
            padding-bottom: 10px;
        }
        .scenario-btn {
            width: 100%;
            padding: 15px;
            margin: 8px 0;
            border: 2px solid #e2e8f0;
            border-radius: 8px;
            background: white;
            cursor: pointer;
            transition: all 0.2s;
            text-align: left;
        }
        .scenario-btn:hover {
            border-color: #667eea;
            background: #f7fafc;
            transform: translateY(-2px);
        }
        .scenario-btn.active {
            border-color: #667eea;
            background: #eef2ff;
        }
        .scenario-title {
            font-weight: 600;
            font-size: 16px;
            color: #1a202c;
            margin-bottom: 4px;
        }
        .scenario-desc {
            font-size: 13px;
            color: #718096;
        }
        .custom-input {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 10px;
            margin-top: 10px;
        }
        .input-group {
            margin-bottom: 10px;
        }
        .input-group label {
            display: block;
            font-size: 12px;
            color: #4a5568;
            margin-bottom: 4px;
            font-weight: 600;
        }
        .input-group input {
            width: 100%;
            padding: 10px;
            border: 2px solid #e2e8f0;
            border-radius: 6px;
            font-size: 14px;
        }
        .btn {
            width: 100%;
            padding: 16px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            margin-top: 10px;
        }
        .btn-primary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .btn-primary:hover:not(:disabled) {
            transform: translateY(-2px);
            box-shadow: 0 10px 30px rgba(102, 126, 234, 0.4);
        }
        .btn-primary:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        .btn-danger {
            background: #ef4444;
            color: white;
        }
        .btn-danger:hover {
            background: #dc2626;
        }
        .progress-container {
            margin: 20px 0;
            display: none;
        }
        .progress-bar {
            width: 100%;
            height: 30px;
            background: #e2e8f0;
            border-radius: 15px;
            overflow: hidden;
            position: relative;
        }
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
            transition: width 0.3s;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-weight: 600;
            font-size: 14px;
        }
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 15px;
            margin: 20px 0;
        }
        .stat-card {
            background: #f7fafc;
            padding: 15px;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }
        .stat-label {
            font-size: 12px;
            color: #718096;
            margin-bottom: 4px;
            text-transform: uppercase;
            font-weight: 600;
        }
        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: #1a202c;
        }
        .log-container {
            background: #1a202c;
            color: #10b981;
            padding: 15px;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 13px;
            max-height: 300px;
            overflow-y: auto;
            margin-top: 15px;
        }
        .log-line {
            margin: 4px 0;
            padding: 2px 0;
        }
        .log-success { color: #10b981; }
        .log-error { color: #ef4444; }
        .log-info { color: #3b82f6; }
        .log-warning { color: #f59e0b; }
        .chart-container {
            margin-top: 20px;
            height: 200px;
        }
        .full-width {
            grid-column: 1 / -1;
        }
        .toggle-switch {
            display: flex;
            align-items: center;
            gap: 10px;
            margin: 15px 0;
        }
        .switch {
            position: relative;
            width: 50px;
            height: 26px;
        }
        .switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: #ccc;
            transition: .4s;
            border-radius: 26px;
        }
        .slider:before {
            position: absolute;
            content: "";
            height: 18px;
            width: 18px;
            left: 4px;
            bottom: 4px;
            background-color: white;
            transition: .4s;
            border-radius: 50%;
        }
        input:checked + .slider {
            background-color: #667eea;
        }
        input:checked + .slider:before {
            transform: translateX(24px);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Load Testing Dashboard</h1>
            <p>Test Kafka 3-Broker Cluster Performance</p>
        </div>

        <div class="grid">
            <!-- Test Configuration -->
            <div class="card">
                <h2>⚙️ Test Configuration</h2>
                
                <div class="toggle-switch">
                    <label class="switch">
                        <input type="checkbox" id="fraudToggle" checked>
                        <span class="slider"></span>
                    </label>
                    <span>Fraud Detection Enabled</span>
                </div>

                <div id="scenarios">
                    <button class="scenario-btn" data-requests="100" data-concurrent="5">
                        <div class="scenario-title">🟢 Light Load</div>
                        <div class="scenario-desc">100 requests, 5 concurrent (~20 req/s)</div>
                    </button>
                    <button class="scenario-btn" data-requests="500" data-concurrent="10">
                        <div class="scenario-title">🟡 Medium Load</div>
                        <div class="scenario-desc">500 requests, 10 concurrent (~50 req/s)</div>
                    </button>
                    <button class="scenario-btn" data-requests="1000" data-concurrent="20">
                        <div class="scenario-title">🟠 Heavy Load</div>
                        <div class="scenario-desc">1000 requests, 20 concurrent (~100 req/s)</div>
                    </button>
                    <button class="scenario-btn" data-requests="5000" data-concurrent="50">
                        <div class="scenario-title">🔴 Stress Test</div>
                        <div class="scenario-desc">5000 requests, 50 concurrent (~500 req/s)</div>
                    </button>
                    <button class="scenario-btn" data-custom="true">
                        <div class="scenario-title">⚙️ Custom</div>
                        <div class="scenario-desc">Configure your own test</div>
                    </button>
                </div>

                <div class="custom-input" id="customInput" style="display: none;">
                    <div class="input-group">
                        <label>Total Requests</label>
                        <input type="number" id="customRequests" value="1000" min="1">
                    </div>
                    <div class="input-group">
                        <label>Concurrent</label>
                        <input type="number" id="customConcurrent" value="10" min="1">
                    </div>
                </div>

                <button class="btn btn-primary" id="startBtn">
                    Start Load Test
                </button>
                <button class="btn btn-danger" id="stopBtn" style="display: none;">
                    Stop Test
                </button>
            </div>

            <!-- Real-time Stats -->
            <div class="card">
                <h2>📊 Real-time Statistics</h2>
                
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-label">Sent</div>
                        <div class="stat-value" id="statSent">0</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">Success</div>
                        <div class="stat-value" id="statSuccess">0</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">Failed</div>
                        <div class="stat-value" id="statFailed">0</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">Rate</div>
                        <div class="stat-value" id="statRate">0%</div>
                    </div>
                </div>

                <div class="progress-container" id="progressContainer">
                    <div class="progress-bar">
                        <div class="progress-fill" id="progressFill">0%</div>
                    </div>
                </div>

                <div class="stats-grid" style="margin-top: 20px;">
                    <div class="stat-card">
                        <div class="stat-label">Throughput</div>
                        <div class="stat-value" id="statThroughput">0</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">Avg Time</div>
                        <div class="stat-value" id="statAvgTime">0ms</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">P95</div>
                        <div class="stat-value" id="statP95">0ms</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">P99</div>
                        <div class="stat-value" id="statP99">0ms</div>
                    </div>
                </div>

                <div class="log-container" id="logContainer">
                    <div class="log-line log-info">Ready to start load test...</div>
                </div>
            </div>
        </div>

        <!-- Analytics Dashboard -->
        <div class="card full-width">
            <h2>📈 Analytics Dashboard</h2>
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-label">Total Payments</div>
                    <div class="stat-value" id="analyticsTotal">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Success Rate</div>
                    <div class="stat-value" id="analyticsSuccess">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Total Revenue</div>
                    <div class="stat-value" id="analyticsRevenue">-</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Fraud Rate</div>
                    <div class="stat-value" id="analyticsFraud">-</div>
                </div>
            </div>
        </div>
    </div>

    <script>
        let testRunning = false;
        let testAborted = false;
        let selectedRequests = 100;
        let selectedConcurrent = 5;
        let stats = {
            sent: 0,
            success: 0,
            failed: 0,
            responseTimes: [],
            startTime: null
        };

        // Scenario selection
        document.querySelectorAll('.scenario-btn').forEach(btn => {
            btn.addEventListener('click', function() {
                document.querySelectorAll('.scenario-btn').forEach(b => b.classList.remove('active'));
                this.classList.add('active');
                
                if (this.dataset.custom) {
                    document.getElementById('customInput').style.display = 'grid';
                    selectedRequests = parseInt(document.getElementById('customRequests').value);
                    selectedConcurrent = parseInt(document.getElementById('customConcurrent').value);
                } else {
                    document.getElementById('customInput').style.display = 'none';
                    selectedRequests = parseInt(this.dataset.requests);
                    selectedConcurrent = parseInt(this.dataset.concurrent);
                }
            });
        });

        // Custom input changes
        document.getElementById('customRequests').addEventListener('change', function() {
            selectedRequests = parseInt(this.value);
        });
        document.getElementById('customConcurrent').addEventListener('change', function() {
            selectedConcurrent = parseInt(this.value);
        });

        // Select first scenario by default
        document.querySelector('.scenario-btn').click();

        // Start test
        document.getElementById('startBtn').addEventListener('click', startLoadTest);
        document.getElementById('stopBtn').addEventListener('click', stopLoadTest);

        async function startLoadTest() {
            if (testRunning) return;
            
            testRunning = true;
            testAborted = false;
            stats = { sent: 0, success: 0, failed: 0, responseTimes: [], startTime: Date.now() };
            
            document.getElementById('startBtn').disabled = true;
            document.getElementById('startBtn').textContent = 'Running...';
            document.getElementById('stopBtn').style.display = 'block';
            document.getElementById('progressContainer').style.display = 'block';
            
            addLog('Starting load test...', 'info');
            addLog(`Configuration: ${selectedRequests} requests, ${selectedConcurrent} concurrent`, 'info');
            
            updateStats();
            
            // Create all promises upfront for true concurrency
            const allPromises = [];
            let activeRequests = 0;
            let completedRequests = 0;
            
            // Function to send request and manage concurrency
            const sendWithConcurrencyLimit = async () => {
                while (completedRequests < selectedRequests && !testAborted) {
                    // Wait if we've hit the concurrency limit
                    while (activeRequests >= selectedConcurrent && !testAborted) {
                        await new Promise(resolve => setTimeout(resolve, 10));
                    }
                    
                    if (testAborted) break;
                    
                    activeRequests++;
                    
                    // Send request without waiting
                    sendPaymentRequest().finally(() => {
                        activeRequests--;
                        completedRequests++;
                        updateStats();
                    });
                    
                    // Small delay to prevent overwhelming the browser
                    await new Promise(resolve => setTimeout(resolve, 1));
                }
            };
            
            // Start sending requests
            await sendWithConcurrencyLimit();
            
            // Wait for all remaining requests to complete
            while (stats.sent < selectedRequests && !testAborted) {
                await new Promise(resolve => setTimeout(resolve, 100));
            }
            
            if (!testAborted) {
                addLog('Load test completed!', 'success');
                addLog(`Total: ${stats.sent}, Success: ${stats.success}, Failed: ${stats.failed}`, 'success');
            } else {
                addLog('Load test aborted by user', 'warning');
            }
            
            testRunning = false;
            document.getElementById('startBtn').disabled = false;
            document.getElementById('startBtn').textContent = 'Start Load Test';
            document.getElementById('stopBtn').style.display = 'none';
            
            // Fetch final analytics
            setTimeout(fetchAnalytics, 2000);
        }

        function stopLoadTest() {
            testAborted = true;
            addLog('Stopping test...', 'warning');
        }

        async function sendPaymentRequest() {
            const startTime = Date.now();
            
            try {
                const response = await fetch('/api/payments', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-CSRF-TOKEN': document.querySelector('meta[name="csrf-token"]').content
                    },
                    body: JSON.stringify({
                        user_id: 1, // Always use user_id = 1
                        amount: Math.floor(Math.random() * 100000) + 1000,
                        currency: ['VND', 'USD', 'EUR'][Math.floor(Math.random() * 3)],
                        payment_method: 'CARD'
                    })
                });
                
                const responseTime = Date.now() - startTime;
                stats.responseTimes.push(responseTime);
                stats.sent++;
                
                if (response.ok) {
                    stats.success++;
                } else {
                    stats.failed++;
                }
            } catch (error) {
                stats.sent++;
                stats.failed++;
                stats.responseTimes.push(Date.now() - startTime);
            }
        }

        function updateStats() {
            document.getElementById('statSent').textContent = stats.sent;
            document.getElementById('statSuccess').textContent = stats.success;
            document.getElementById('statFailed').textContent = stats.failed;
            
            const successRate = stats.sent > 0 ? (stats.success / stats.sent * 100).toFixed(1) : 0;
            document.getElementById('statRate').textContent = successRate + '%';
            
            const progress = (stats.sent / selectedRequests * 100).toFixed(0);
            document.getElementById('progressFill').style.width = progress + '%';
            document.getElementById('progressFill').textContent = progress + '%';
            
            if (stats.responseTimes.length > 0) {
                const sorted = [...stats.responseTimes].sort((a, b) => a - b);
                const avg = sorted.reduce((a, b) => a + b, 0) / sorted.length;
                const p95 = sorted[Math.floor(sorted.length * 0.95)];
                const p99 = sorted[Math.floor(sorted.length * 0.99)];
                
                document.getElementById('statAvgTime').textContent = Math.round(avg) + 'ms';
                document.getElementById('statP95').textContent = Math.round(p95) + 'ms';
                document.getElementById('statP99').textContent = Math.round(p99) + 'ms';
                
                const duration = (Date.now() - stats.startTime) / 1000;
                const throughput = (stats.sent / duration).toFixed(1);
                document.getElementById('statThroughput').textContent = throughput + ' req/s';
            }
        }

        function addLog(message, type = 'info') {
            const logContainer = document.getElementById('logContainer');
            const logLine = document.createElement('div');
            logLine.className = `log-line log-${type}`;
            logLine.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            logContainer.appendChild(logLine);
            logContainer.scrollTop = logContainer.scrollHeight;
            
            // Keep only last 50 lines
            while (logContainer.children.length > 50) {
                logContainer.removeChild(logContainer.firstChild);
            }
        }

        async function fetchAnalytics() {
            try {
                const response = await fetch('/api/analytics/dashboard');
                const data = await response.json();
                
                if (data.success && data.data.today) {
                    const today = data.data.today;
                    document.getElementById('analyticsTotal').textContent = today.total_payments;
                    document.getElementById('analyticsSuccess').textContent = today.success_rate + '%';
                    document.getElementById('analyticsRevenue').textContent = new Intl.NumberFormat().format(today.total_revenue);
                    document.getElementById('analyticsFraud').textContent = today.fraud_rate + '%';
                }
            } catch (error) {
                console.error('Failed to fetch analytics:', error);
            }
        }

        // Fetch analytics on load and every 5 seconds
        fetchAnalytics();
        setInterval(fetchAnalytics, 5000);

        // Fraud detection toggle
        document.getElementById('fraudToggle').addEventListener('change', function() {
            addLog(this.checked ? 'Fraud detection enabled' : 'Fraud detection disabled', 'info');
            // Note: This would need backend implementation to actually toggle fraud detection
        });
    </script>
</body>
</html>
