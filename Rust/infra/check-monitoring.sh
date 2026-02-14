#!/bin/bash

echo "=== Checking Monitoring Stack ==="
echo ""

echo "1. Checking Docker containers..."
docker-compose ps

echo ""
echo "2. Checking Prometheus targets..."
curl -s http://localhost:9090/api/v1/targets | grep -o '"health":"[^"]*"' | head -20

echo ""
echo "3. Checking available metrics..."
echo "Node Exporter metrics:"
curl -s http://localhost:9100/metrics | grep "node_cpu_seconds_total" | head -3

echo ""
echo "Redis Exporter metrics:"
curl -s http://localhost:9121/metrics | grep "redis_up"

echo ""
echo "cAdvisor metrics:"
curl -s http://localhost:8888/metrics | grep "container_cpu_usage_seconds_total" | head -3

echo ""
echo "4. Test Prometheus queries..."
echo "CPU query:"
curl -s 'http://localhost:9090/api/v1/query?query=node_cpu_seconds_total' | grep -o '"status":"[^"]*"'

echo ""
echo "Memory query:"
curl -s 'http://localhost:9090/api/v1/query?query=node_memory_MemTotal_bytes' | grep -o '"status":"[^"]*"'

echo ""
echo "Redis query:"
curl -s 'http://localhost:9090/api/v1/query?query=redis_up' | grep -o '"status":"[^"]*"'

echo ""
echo "=== Check complete ==="
echo "Visit http://localhost:9090/targets to see all targets"
