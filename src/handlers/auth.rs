// Authentication handlers (register, login)

use axum::{extract::State, http::StatusCode, response::Json, Extension};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use tracing::{info, warn, error, debug, instrument};

use crate::api_models::{ApiResponse, LoginRequest, LoginResponse, RegisterRequest, UserResponse};
use crate::auth::{hash_password, verify_password, Claims};
use crate::middleware::AppState;
use crate::models::User;

type ErrorResponse = (StatusCode, Json<ApiResponse<()>>);

// Helper function to create error responses
fn error_response(status: StatusCode, message: String) -> ErrorResponse {
    (status, Json(ApiResponse::<()>::error(message)))
}

// Helper function to convert User to UserResponse
fn user_to_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        is_active: user.is_active.unwrap_or(true),
        is_verified: user.is_verified.unwrap_or(false),
        created_at: user.created_at.unwrap_or_else(|| Utc::now()),
    }
}

#[instrument(skip(state, payload), fields(email = %payload.email, username = %payload.username))]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, ErrorResponse> {
    info!("Processing user registration request");

    // Validate input
    if let Err(errors) = payload.validate() {
        warn!("Registration validation failed: {:?}", errors);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    debug!("Checking for existing user");
    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 OR username = $2",
        payload.email,
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error while checking existing user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_user.is_some() {
        warn!("Registration attempt with existing email or username");
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("User already exists with this email or username".to_string())),
        ));
    }

    debug!("Hashing password");
    // Hash password
    let password_hash = hash_password(&payload.password).map_err(|e| {
        error!("Password hashing error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Password hashing error: {}", e))),
        )
    })?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    info!("Creating new user with ID: {}", user_id);
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, username, password_hash, first_name, last_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        user_id,
        payload.email,
        payload.username,
        password_hash,
        payload.first_name,
        payload.last_name,
        now,
        now
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to create user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to create user: {}", e))),
        )
    })?;

    // Fetch created user
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch created user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to fetch created user: {}", e))),
            )
        })?;

    let user_response = user_to_response(user);
    info!("User registration successful");

    Ok(Json(ApiResponse::success(user_response)))
}

#[instrument(skip(state, payload), fields(email = %payload.email))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ErrorResponse> {
    info!("Processing login request");

    // Validate input
    if let Err(errors) = payload.validate() {
        warn!("Login validation failed: {:?}", errors);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    debug!("Looking up user by email");
    // Find user by email
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error during login: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        warn!("Login attempt with non-existent email");
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Invalid credentials".to_string())),
        )
    })?;

    // Check if user is active
    if !user.is_active.unwrap_or(false) {
        warn!("Login attempt for inactive user: {}", user.id);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("User account is not active".to_string())),
        ));
    }

    debug!("Verifying password");
    // Verify password
    let password_valid = verify_password(&payload.password, &user.password_hash).map_err(|e| {
        error!("Password verification error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Password verification error: {}", e))),
        )
    })?;

    if !password_valid {
        warn!("Login attempt with invalid password for user: {}", user.id);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Invalid credentials".to_string())),
        ));
    }

    debug!("Getting user's organization membership");
    // Get user's primary organization (if any)
    let org_id = sqlx::query!(
        "SELECT organization_id FROM organization_memberships WHERE user_id = $1 AND is_active = true ORDER BY created_at ASC LIMIT 1",
        user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error fetching organization: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .map(|row| row.organization_id);

    debug!("Creating JWT session token");
    // Create JWT token for session
    let claims = Claims::new(user.id, user.email.clone(), org_id);
    let token = state.jwt_manager.create_token(&claims).map_err(|e| {
        error!("Token creation failed: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Token creation failed: {:?}", e))),
        )
    })?;

    // Create API key for Redis operations (if user has organization)
    let api_key = if let Some(org_id) = org_id {
        debug!("Creating API key for organization: {}", org_id);
        // Generate API key ID and prefix
        let api_key_id = Uuid::new_v4();
        let key_prefix = format!("rg_{}", &api_key_id.to_string().replace("-", "")[..12]);

        // Create API key claims with full permissions
        let api_key_claims = crate::auth::ApiKeyClaims::new(
            api_key_id,
            user.id,
            org_id,
            vec!["*".to_string()], // Full permissions
            key_prefix.clone(),
            None, // No expiration
        );

        // Generate JWT token for API key
        let api_key_token = state.jwt_manager.create_api_key_token(&api_key_claims).map_err(|e| {
            error!("API key token creation failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("API key creation failed: {:?}", e))),
            )
        })?;

        // Save API key to database (store JWT token directly)
        let now = Utc::now();
        sqlx::query!(
            r#"
            INSERT INTO api_keys (id, name, key_token, key_prefix, user_id, organization_id, scopes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            api_key_id,
            format!("Auto-generated key for {}", user.email),
            api_key_token,
            key_prefix,
            user.id,
            org_id,
            &vec!["*".to_string()],
            now,
            now
        )
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to save API key to database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to save API key: {}", e))),
            )
        })?;

        info!("API key created successfully for user: {}", user.id);
        Some(api_key_token)
    } else {
        debug!("No organization found, skipping API key creation");
        None
    };

    let user_response = user_to_response(user);

    let login_response = LoginResponse {
        token,
        user: user_response,
        api_key,
        organization_id: org_id,
    };

    info!("Login successful for user");
    Ok(Json(ApiResponse::success(login_response)))
}

/// Get current authenticated user information
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<crate::middleware::CurrentUser>,
) -> Result<Json<ApiResponse<UserResponse>>, ErrorResponse> {
    // Fetch full user data from database
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        current_user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to fetch user: {}", e))),
        )
    })?;

    let user_response = user_to_response(user);
    Ok(Json(ApiResponse::success(user_response)))
}
