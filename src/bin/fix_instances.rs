use sqlx::postgres::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://redisgate_dev:devpassword123@127.0.0.1:5432/redisgate_dev".to_string());

    println!("🔗 Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    println!("📝 Updating instances to use localhost Redis...");

    // Instead of setting domain, we'll clear it so code falls back to localhost
    let result = sqlx::query!(
        r#"
        UPDATE redis_instances
        SET domain = NULL,
            service_name = NULL,
            public_ip_address = NULL,
            private_ip_address = NULL
        WHERE deleted_at IS NULL
        "#
    )
    .execute(&pool)
    .await?;

    println!("✅ Updated {} instances to use localhost fallback", result.rows_affected());

    // Verify
    println!("\n📋 Current instances:");
    let instances = sqlx::query!(
        r#"
        SELECT id, name, slug, domain, port
        FROM redis_instances
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 5
        "#
    )
    .fetch_all(&pool)
    .await?;

    for inst in instances {
        println!("  • {} → {}:{}", inst.name, inst.domain.as_deref().unwrap_or("N/A"), inst.port);
    }

    pool.close().await;
    Ok(())
}

