use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: Uuid,
    email: String,
    org_id: Option<Uuid>,
    exp: i64,
    iat: i64,
}

fn main() {
    let token = std::env::args().nth(1).expect("Please provide token as argument");
    let secret = "default-secret-key";

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // Don't validate expiration for debugging

    match decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
        Ok(token_data) => {
            println!("✓ Token is valid!");
            println!("User ID: {}", token_data.claims.user_id);
            println!("Email: {}", token_data.claims.email);
            println!("Org ID: {:?}", token_data.claims.org_id);
            println!("Issued at: {} ({})", token_data.claims.iat,
                chrono::NaiveDateTime::from_timestamp_opt(token_data.claims.iat, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "Invalid".to_string()));
            println!("Expires at: {} ({})", token_data.claims.exp,
                chrono::NaiveDateTime::from_timestamp_opt(token_data.claims.exp, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "Invalid".to_string()));

            let now = chrono::Utc::now().timestamp();
            if token_data.claims.exp < now {
                println!("\n⚠️  TOKEN HAS EXPIRED!");
                println!("Current time: {} ({})", now,
                    chrono::NaiveDateTime::from_timestamp_opt(now, 0)
                        .map(|dt| dt.to_string())
                        .unwrap_or_else(|| "Invalid".to_string()));
            } else {
                println!("\n✓ Token is still valid");
            }
        }
        Err(e) => {
            println!("✗ Token is invalid: {:?}", e);
        }
    }
}

