// Redis instance management handlers

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use sqlx::{Row, types::BigDecimal};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use tracing::{info, warn, error, debug, instrument};

use crate::api_models::{
    ApiResponse, CreateRedisInstanceRequest, PaginatedResponse, PaginationParams,
    RedisInstanceResponse,
};
use crate::auth::hash_password;
use crate::k8s_service::K8sRedisService;
use crate::middleware::{AppState, CurrentUser};
use crate::models::RedisInstance;

type ErrorResponse = (StatusCode, Json<ApiResponse<()>>);

// Helper function to convert RedisInstance to RedisInstanceResponse
fn redis_instance_to_response(redis_instance: RedisInstance) -> RedisInstanceResponse {
    RedisInstanceResponse {
        id: redis_instance.id,
        name: redis_instance.name,
        slug: redis_instance.slug,
        organization_id: redis_instance.organization_id,
        api_key_id: redis_instance.api_key_id,
        port: redis_instance.port.unwrap_or(6379),
        domain: redis_instance.domain,
        max_memory: redis_instance.max_memory.unwrap_or(0),
        current_memory: redis_instance.current_memory.unwrap_or(0),
        redis_version: redis_instance.redis_version.unwrap_or_else(|| "7.0".to_string()),
        namespace: redis_instance.namespace.unwrap_or_else(|| "default".to_string()),
        status: redis_instance.status.unwrap_or_else(|| "unknown".to_string()),
        health_status: redis_instance.health_status.unwrap_or_else(|| "unknown".to_string()),
        cpu_usage_percent: redis_instance.cpu_usage_percent
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        memory_usage_percent: redis_instance.memory_usage_percent
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        connections_count: redis_instance.connections_count.unwrap_or(0),
        max_connections: redis_instance.max_connections.unwrap_or(1000),
        persistence_enabled: redis_instance.persistence_enabled.unwrap_or(false),
        backup_enabled: redis_instance.backup_enabled.unwrap_or(false),
        last_backup_at: redis_instance.last_backup_at,
        created_at: redis_instance.created_at.unwrap_or_else(|| Utc::now()),
        updated_at: redis_instance.updated_at.unwrap_or_else(|| Utc::now()),
    }
}

// Generate a secure Redis password
fn generate_redis_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    
    (0..24)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn create_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateRedisInstanceRequest>,
) -> Result<Json<ApiResponse<RedisInstanceResponse>>, ErrorResponse> {
    // Validate input
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation error: {:?}", errors))),
        ));
    }

    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        payload.organization_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Check quota limits using QuotaService
    use crate::services::quota::{QuotaService, QuotaError};
    let quota_service = QuotaService::new(Arc::new(state.db_pool.clone()));
    let memory_mb = (payload.max_memory / 1024 / 1024) as i32; // Convert bytes to MB

    if let Err(e) = quota_service.check_can_create_instance(payload.organization_id, memory_mb).await {
        let (status, message) = match e {
            QuotaError::MaxInstancesReached { current, max } => (
                StatusCode::FORBIDDEN,
                format!("Organization has reached the maximum number of Redis instances ({}/{}). Please upgrade your plan or delete unused instances.", current, max)
            ),
            QuotaError::MemoryLimitExceeded { requested, available, total_gb } => (
                StatusCode::FORBIDDEN,
                format!("Memory limit exceeded: requested {}MB, only {}MB available out of {}GB total. Please reduce memory or upgrade your plan.", requested, available, total_gb)
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Quota check failed: {}", e)
            ),
        };
        return Err((status, Json(ApiResponse::<()>::error(message))));
    }

    // Check if slug is unique within organization
    let existing_instance = sqlx::query!(
        "SELECT id FROM redis_instances WHERE organization_id = $1 AND slug = $2 AND deleted_at IS NULL",
        payload.organization_id,
        payload.slug
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    if existing_instance.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Redis instance with this slug already exists in the organization".to_string())),
        ));
    }

    // Create Redis instance without automatic API key creation
    let _instance_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Generate Redis password and hash it
    let redis_password = generate_redis_password();
    let redis_password_hash = hash_password(&redis_password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Password hashing error: {}", e))),
        )
    })?;

    // Create Redis instance
    let instance_id = Uuid::new_v4();
    let redis_version = payload.redis_version.unwrap_or_else(|| "7.2".to_string());
    let persistence_enabled = payload.persistence_enabled.unwrap_or(true);
    let backup_enabled = payload.backup_enabled.unwrap_or(false);
    let namespace = format!("redis-{}", payload.organization_id.simple());
    
    // For development mode, use port 6379 (shared Redis) with unique slug-based database
    // For production K8s, each instance gets its own port
    let port = 6379;

    // Use slug as domain to ensure uniqueness in development mode
    let domain = format!("localhost-{}", payload.slug);

    // Try to deploy to Kubernetes if available (silent fallback to simulation mode)
    let k8s_deployment_result = match crate::k8s_service::K8sRedisService::new().await {
        Ok(k8s_service) => {
            let config = crate::k8s_service::RedisDeploymentConfig {
                name: payload.name.clone(),
                slug: payload.slug.clone(),
                namespace: namespace.clone(),
                organization_id: payload.organization_id,
                instance_id,
                redis_version: redis_version.clone(),
                max_memory: payload.max_memory,
                redis_password: redis_password.clone(),
                port,
            };
            
            match k8s_service.create_redis_instance(config).await {
                Ok(result) => {
                    tracing::info!("Redis instance deployed to Kubernetes: {}", instance_id);
                    Some(result)
                },
                Err(e) => {
                    tracing::debug!("K8s deployment skipped: {}. Using simulation mode.", e);
                    None
                }
            }
        },
        Err(_) => {
            // K8s not available - silent fallback to simulation mode
            tracing::debug!("Running in simulation mode (K8s not configured)");
            None
        }
    };

    // Use deployment result or mock data for database record
    let (actual_port, actual_domain, actual_namespace, deployment_name, service_name, status) = 
        if let Some(ref result) = k8s_deployment_result {
            (
                result.port,
                result.domain.clone(),
                result.namespace.clone(),
                result.deployment_name.clone(),
                result.service_name.clone(),
                "pending" // K8s deployment is pending
            )
        } else {
            // Development mode - use localhost Redis with NULL domain (avoids unique constraint)
            // We'll use private_ip_address instead to store 127.0.0.1
            tracing::info!("Creating instance in development mode, using localhost:6379 (slug: {})", payload.slug);
            (
                6379,
                format!("dev-{}", payload.slug), // Unique domain per instance
                namespace.clone(),
                format!("redis-{}", payload.slug),
                format!("redis-{}-service", payload.slug),
                "development" // Development mode with local Redis
            )
        };

    sqlx::query(
        r#"
        INSERT INTO redis_instances (
            id, name, slug, organization_id, port, domain,
            max_memory, current_memory, password_hash, redis_version, namespace,
            pod_name, service_name, status, health_status, cpu_usage_percent, memory_usage_percent,
            connections_count, max_connections, persistence_enabled, backup_enabled,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
        "#,
    )
    .bind(instance_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(payload.organization_id)
    .bind(actual_port)
    .bind(&actual_domain)
    .bind(payload.max_memory)
    .bind(0i64) // current_memory starts at 0
    .bind(&redis_password_hash)
    .bind(&redis_version)
    .bind(&actual_namespace)
    .bind(&deployment_name) // pod_name (using deployment name)
    .bind(&service_name)
    .bind(status) // status reflects K8s deployment state
    .bind("unknown") // health_status
    .bind(BigDecimal::new(0.into(), 2)) // cpu_usage_percent
    .bind(BigDecimal::new(0.into(), 2)) // memory_usage_percent
    .bind(0i32) // connections_count
    .bind(100i32) // max_connections (default)
    .bind(persistence_enabled)
    .bind(backup_enabled)
    .bind(now)
    .bind(now)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        // If database insert fails, we should clean up K8s resources if they were created
        if let Some(result) = k8s_deployment_result {
            tokio::spawn(async move {
                if let Ok(k8s_service) = crate::k8s_service::K8sRedisService::new().await {
                    let _ = k8s_service.delete_redis_instance(&result.namespace, &payload.slug).await;
                }
            });
        }
        
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to create Redis instance: {}", e))),
        )
    })?;

    // Fetch created instance
    let redis_instance = sqlx::query_as!(
        RedisInstance,
        "SELECT * FROM redis_instances WHERE id = $1",
        instance_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to fetch created Redis instance: {}", e))),
        )
    })?;

    let instance_response = redis_instance_to_response(redis_instance);

    Ok(Json(ApiResponse::success(instance_response)))
}

pub async fn list_redis_instances(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<PaginationParams>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PaginatedResponse<RedisInstanceResponse>>>, ErrorResponse> {
    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Get Redis instances for the organization
    let redis_instances = sqlx::query_as!(
        RedisInstance,
        r#"
        SELECT * FROM redis_instances 
        WHERE organization_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        org_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?;

    // Get total count
    let total_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM redis_instances WHERE organization_id = $1 AND deleted_at IS NULL",
        org_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .count
    .unwrap_or(0);

    let instance_responses: Vec<RedisInstanceResponse> = redis_instances
        .into_iter()
        .map(redis_instance_to_response)
        .collect();

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    let paginated_response = PaginatedResponse {
        items: instance_responses,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(Json(ApiResponse::success(paginated_response)))
}

pub async fn get_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<RedisInstanceResponse>>, ErrorResponse> {
    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Get Redis instance
    let redis_instance = sqlx::query_as!(
        RedisInstance,
        "SELECT * FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        instance_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Redis instance not found".to_string())),
        )
    })?;

    let instance_response = redis_instance_to_response(redis_instance);

    Ok(Json(ApiResponse::success(instance_response)))
}

pub async fn delete_redis_instance(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, ErrorResponse> {
    // Check if user has admin access to the organization
    let org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    if !["admin", "owner"].contains(&org_membership.role.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Insufficient permissions to delete Redis instances".to_string())),
        ));
    }

    // Check if Redis instance exists and get its details
    let redis_instance = sqlx::query!(
        "SELECT namespace, slug, api_key_id FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        instance_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Redis instance not found".to_string())),
        )
    })?;

    let now = Utc::now();

    let namespace = redis_instance.namespace;
    let slug = redis_instance.slug;
    let api_key_id = redis_instance.api_key_id;

    // Try to delete from Kubernetes if available
    match K8sRedisService::new().await {
        Ok(k8s_service) => {
            if let Err(e) = k8s_service.delete_redis_instance(&namespace, &slug).await {
                tracing::warn!("Failed to delete Redis from Kubernetes: {}. Continuing with database deletion.", e);
            }
        }
        Err(e) => {
            tracing::warn!("Kubernetes not available: {}. Skipping K8s deletion.", e);
        }
    }

    // Soft delete Redis instance
    sqlx::query!(
        "UPDATE redis_instances SET deleted_at = $1, updated_at = $2 WHERE id = $3",
        now,
        now,
        instance_id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to delete Redis instance: {}", e))),
        )
    })?;

    // Deactivate associated API key if it exists
    if let Some(key_id) = api_key_id {
        sqlx::query!(
            "UPDATE api_keys SET is_active = false, updated_at = $1 WHERE id = $2",
            now,
            key_id
        )
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to deactivate API key: {}", e))),
            )
        })?;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Redis instance deleted successfully".to_string()),
        timestamp: Utc::now(),
    }))
}

pub async fn update_redis_instance_status(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
    Path((org_id, instance_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<RedisInstanceResponse>>, ErrorResponse> {
    // Check if user has access to the organization
    let _org_membership = sqlx::query!(
        r#"
        SELECT role FROM organization_memberships 
        WHERE organization_id = $1 AND user_id = $2 AND is_active = true
        "#,
        org_id,
        current_user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Organization not found or access denied".to_string())),
        )
    })?;

    // Get Redis instance
    let redis_instance = sqlx::query(
        "SELECT namespace, slug, status FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
    )
    .bind(instance_id)
    .bind(org_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Redis instance not found".to_string())),
        )
    })?;

    // Check Kubernetes deployment status
    let namespace: Option<String> = redis_instance.try_get("namespace").ok();
    let slug: Option<String> = redis_instance.try_get("slug").ok();
    let current_status: Option<String> = redis_instance.try_get("status").ok();

    if let (Some(namespace), Some(slug)) = (&namespace, &slug) {
        match K8sRedisService::new().await {
            Ok(k8s_service) => {
                match k8s_service.get_deployment_status(namespace, slug).await {
                    Ok(k8s_status) => {
                        // Update status in database if it changed
                        if current_status.as_deref() != Some(&k8s_status) {
                            sqlx::query(
                                "UPDATE redis_instances SET status = $1, updated_at = $2 WHERE id = $3",
                            )
                            .bind(&k8s_status)
                            .bind(chrono::Utc::now())
                            .bind(instance_id)
                            .execute(&state.db_pool)
                            .await
                            .map_err(|e| {
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ApiResponse::<()>::error(format!("Failed to update status: {}", e))),
                                )
                            })?;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to check Kubernetes status: {}. Using database status.", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Kubernetes not available: {}. Using database status.", e);
            }
        }
    }

    // Fetch updated instance
    let updated_instance = sqlx::query_as!(
        RedisInstance,
        "SELECT * FROM redis_instances WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        instance_id,
        org_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Redis instance not found".to_string())),
        )
    })?;

    let instance_response = redis_instance_to_response(updated_instance);

    Ok(Json(ApiResponse::success(instance_response)))
}
