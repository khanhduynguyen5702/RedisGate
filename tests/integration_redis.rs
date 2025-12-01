/// Integration tests with real Redis instance
use redis::{Client, Commands};
use std::env;

fn get_redis_url() -> String {
    env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

fn setup_redis() -> Result<redis::Connection, redis::RedisError> {
    let client = Client::open(get_redis_url().as_str())?;
    client.get_connection()
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_redis_ping() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();
        let result: String = redis::cmd("PING").query(&mut conn).unwrap();
        assert_eq!(result, "PONG");
        println!("✓ PING test passed");
    }

    #[test]
    fn test_redis_set_get() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // SET
        let _: () = conn.set("test_key", "test_value").unwrap();

        // GET
        let result: String = conn.get("test_key").unwrap();
        assert_eq!(result, "test_value");

        // Cleanup
        let _: () = conn.del("test_key").unwrap();
        println!("✓ SET/GET test passed");
    }

    #[test]
    fn test_redis_incr_decr() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // SET initial value
        let _: () = conn.set("counter", 0).unwrap();

        // INCR
        let result: i64 = conn.incr("counter", 1).unwrap();
        assert_eq!(result, 1);

        let result: i64 = conn.incr("counter", 1).unwrap();
        assert_eq!(result, 2);

        // DECR
        let result: i64 = conn.decr("counter", 1).unwrap();
        assert_eq!(result, 1);

        // Cleanup
        let _: () = conn.del("counter").unwrap();
        println!("✓ INCR/DECR test passed");
    }

    #[test]
    fn test_redis_expire_ttl() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // SET key
        let _: () = conn.set("temp_key", "temp_value").unwrap();

        // EXPIRE (10 seconds)
        let result: i32 = redis::cmd("EXPIRE")
            .arg("temp_key")
            .arg(10)
            .query(&mut conn)
            .unwrap();
        assert_eq!(result, 1); // 1 means expiration was set

        // TTL
        let ttl: i64 = redis::cmd("TTL")
            .arg("temp_key")
            .query(&mut conn)
            .unwrap();
        assert!(ttl > 0 && ttl <= 10);

        // Cleanup
        let _: () = conn.del("temp_key").unwrap();
        println!("✓ EXPIRE/TTL test passed");
    }

    #[test]
    fn test_redis_exists() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // SET key
        let _: () = conn.set("exists_key", "value").unwrap();

        // EXISTS - should return 1
        let result: i32 = redis::cmd("EXISTS")
            .arg("exists_key")
            .query(&mut conn)
            .unwrap();
        assert_eq!(result, 1);

        // DELETE
        let _: () = conn.del("exists_key").unwrap();

        // EXISTS - should return 0
        let result: i32 = redis::cmd("EXISTS")
            .arg("exists_key")
            .query(&mut conn)
            .unwrap();
        assert_eq!(result, 0);

        println!("✓ EXISTS test passed");
    }

    #[test]
    fn test_redis_hset_hget() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // HSET
        let _: i32 = conn.hset("user:1", "name", "Alice").unwrap();
        let _: i32 = conn.hset("user:1", "age", "25").unwrap();

        // HGET
        let name: String = conn.hget("user:1", "name").unwrap();
        assert_eq!(name, "Alice");

        let age: String = conn.hget("user:1", "age").unwrap();
        assert_eq!(age, "25");

        // Cleanup
        let _: () = conn.del("user:1").unwrap();
        println!("✓ HSET/HGET test passed");
    }

    #[test]
    fn test_redis_lpush_lpop() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // LPUSH
        let _: i32 = conn.lpush("queue", "task1").unwrap();
        let _: i32 = conn.lpush("queue", "task2").unwrap();
        let _: i32 = conn.lpush("queue", "task3").unwrap();

        // LPOP
        let result: String = conn.lpop("queue", None).unwrap();
        assert_eq!(result, "task3"); // Last in, first out

        let result: String = conn.lpop("queue", None).unwrap();
        assert_eq!(result, "task2");

        // Cleanup
        let _: () = conn.del("queue").unwrap();
        println!("✓ LPUSH/LPOP test passed");
    }

    #[test]
    fn test_full_workflow() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping test");
            return;
        }

        let mut conn = conn.unwrap();

        // 1. SET with expiration
        let _: () = conn.set("workflow_key", "value").unwrap();
        let _: i32 = redis::cmd("EXPIRE")
            .arg("workflow_key")
            .arg(300)
            .query(&mut conn)
            .unwrap();

        // 2. Verify EXISTS
        let exists: i32 = redis::cmd("EXISTS")
            .arg("workflow_key")
            .query(&mut conn)
            .unwrap();
        assert_eq!(exists, 1);

        // 3. Check TTL
        let ttl: i64 = redis::cmd("TTL")
            .arg("workflow_key")
            .query(&mut conn)
            .unwrap();
        assert!(ttl > 0);

        // 4. GET value
        let value: String = conn.get("workflow_key").unwrap();
        assert_eq!(value, "value");

        // 5. DELETE
        let deleted: i32 = conn.del("workflow_key").unwrap();
        assert_eq!(deleted, 1);

        // 6. Verify deleted
        let exists: i32 = redis::cmd("EXISTS")
            .arg("workflow_key")
            .query(&mut conn)
            .unwrap();
        assert_eq!(exists, 0);

        println!("✓ Full workflow test passed");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_set_get() {
        let conn = setup_redis();
        if conn.is_err() {
            println!("⚠️  Redis not available, skipping benchmark");
            return;
        }

        let mut conn = conn.unwrap();
        let iterations = 1000;

        let start = Instant::now();
        for i in 0..iterations {
            let key = format!("bench_key_{}", i);
            let _: () = conn.set(&key, "value").unwrap();
            let _: String = conn.get(&key).unwrap();
            let _: () = conn.del(&key).unwrap();
        }
        let duration = start.elapsed();

        let ops_per_sec = (iterations as f64 / duration.as_secs_f64()) as u64;
        println!("✓ Benchmark: {} ops/sec ({} iterations in {:?})",
                 ops_per_sec, iterations, duration);

        assert!(ops_per_sec > 100, "Performance too low: {} ops/sec", ops_per_sec);
    }
}

