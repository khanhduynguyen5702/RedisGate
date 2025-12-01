@echo off
cd /d K:\RedisGate
set RUST_LOG=debug
cargo run --bin redisgate
pause

