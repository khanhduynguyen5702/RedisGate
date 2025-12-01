use sqlx::postgres::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://redisgate_dev:devpassword123@127.0.0.1:5432/redisgate_dev".to_string());

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    println!("\n=== Redis Instances ===");
    let instances = sqlx::query!(
        r#"
        SELECT id, name, slug, service_name, domain, port, password_hash
        FROM redis_instances
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(&pool)
    .await?;

    if instances.is_empty() {
        println!("No instances found!");
    } else {
        for (idx, inst) in instances.iter().enumerate() {
            println!("\n{}. Instance: {}", idx + 1, inst.name);
            println!("   ID: {}", inst.id);
            println!("   Slug: {}", inst.slug);
            println!("   Service Name: {:?}", inst.service_name);
            println!("   Domain: {:?}", inst.domain);
            println!("   Port: {}", inst.port);
            println!("   Has Password: {}", inst.password_hash.is_some());

            // Determine connection info
            let host = if let Some(domain) = &inst.domain {
                domain.clone()
            } else if let Some(service_name) = &inst.service_name {
                service_name.clone()
            } else {
                "127.0.0.1 (fallback)".to_string()
            };

            println!("   → Will connect to: {}:{}", host, inst.port);
        }
    }

    pool.close().await;
    Ok(())
}

