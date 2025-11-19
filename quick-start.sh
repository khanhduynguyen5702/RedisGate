#!/bin/bash
# RedisGate - Quick Start Script

set -e

echo "============================================"
echo "RedisGate - Quick Start Script"
echo "============================================"
echo ""

echo "Step 1: Starting Docker services..."
docker-compose up -d
echo "✓ Docker services started"
echo ""

echo "Step 2: Waiting for services to be ready..."
sleep 5

echo "Step 3: Testing PostgreSQL connection..."
max_attempts=12
attempt=0
while [ $attempt -lt $max_attempts ]; do
    if psql postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev -c "SELECT 1;" > /dev/null 2>&1; then
        echo "✓ PostgreSQL is ready"
        break
    fi
    attempt=$((attempt + 1))
    echo "  Waiting for PostgreSQL... ($attempt/$max_attempts)"
    sleep 5
done

if [ $attempt -eq $max_attempts ]; then
    echo "✗ PostgreSQL failed to start"
    exit 1
fi
echo ""

echo "Step 4: Running database migrations..."
sqlx migrate run
echo "✓ Migrations completed"
echo ""

echo "Step 5: Testing Redis connection..."
if redis-cli -h localhost -p 6379 PING > /dev/null 2>&1; then
    echo "✓ Redis is accessible"
else
    echo "⚠ Redis is not responding (this is OK if not installed locally)"
fi
echo ""

echo "Step 6: Building RedisGate..."
cargo build --release
echo "✓ Build successful"
echo ""

echo "Step 7: Testing instance connections..."
cargo run --bin test_connections
echo ""

echo "============================================"
echo "RedisGate is ready!"
echo "============================================"
echo ""
echo "Next steps:"
echo "  1. Start the server: cargo run"
echo "  2. Register a user: POST http://localhost:8080/auth/register"
echo "  3. Create an organization: POST http://localhost:8080/api/organizations"
echo "  4. Create a Redis instance: POST http://localhost:8080/api/organizations/{org_id}/redis-instances"
echo ""
echo "Documentation:"
echo "  - README.md - Overview and API documentation"
echo "  - INSTANCE_CONNECTION_GUIDE.md - Troubleshooting connections"
echo "  - DEVELOPMENT.md - Development setup details"
echo ""

