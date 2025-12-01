#!/bin/bash
# Test script to verify Redis instance connection

echo "=== Testing Redis Instance Connection ==="
echo ""

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL not set, loading from env.development"
    export $(cat env.development | grep DATABASE_URL | xargs)
fi

echo "1. Testing database connection..."
psql "$DATABASE_URL" -c "SELECT COUNT(*) as instance_count FROM redis_instances WHERE deleted_at IS NULL;" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✓ Database connection successful"
else
    echo "✗ Database connection failed"
    echo "Please ensure PostgreSQL is running and DATABASE_URL is correct"
    exit 1
fi

echo ""
echo "2. Listing Redis instances..."
psql "$DATABASE_URL" -c "SELECT id, name, slug, domain, port, status FROM redis_instances WHERE deleted_at IS NULL;" 2>/dev/null

echo ""
echo "3. Testing Redis server connection (localhost:6379)..."
if command -v redis-cli &> /dev/null; then
    redis-cli -h localhost -p 6379 PING 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✓ Redis server is running on localhost:6379"
    else
        echo "✗ Redis server is not accessible on localhost:6379"
        echo "Please start Redis server: redis-server"
    fi
else
    echo "⚠️  redis-cli not found, skipping Redis server test"
fi

echo ""
echo "=== Connection Test Complete ==="

