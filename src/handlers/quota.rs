use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api_models::ApiResponse;
use crate::middleware::{AppState, CurrentUser};
use crate::services::quota::{QuotaError, QuotaInfo, QuotaService};

type ErrorResponse = (StatusCode, Json<ApiResponse<()>>);

#[derive(Debug, Serialize)]
pub struct QuotaResponse {
    pub quota: QuotaInfo,
    pub warnings: Vec<String>,
}

/// Get quota information for an organization
pub async fn get_quota(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<ApiResponse<QuotaResponse>>, ErrorResponse> {
    // Verify user is member of organization
    let membership = sqlx::query!(
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
    })?;

    if membership.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Access denied to this organization".to_string())),
        ));
    }

    // Get quota information
    let quota_service = QuotaService::new(Arc::new(state.db_pool.clone()));
    let quota = quota_service.get_quota_info(org_id).await.map_err(|e| {
        let status = match e {
            QuotaError::OrgNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ApiResponse::<()>::error(format!("{}", e))))
    })?;

    // Generate warnings
    let mut warnings = Vec::new();

    if quota.instances_percentage >= 90.0 {
        warnings.push(format!(
            "You are using {}% of your instance quota ({}/{})",
            quota.instances_percentage as i32,
            quota.current_instances,
            quota.max_instances
        ));
    }

    if quota.memory_percentage >= 90.0 {
        warnings.push(format!(
            "You are using {}% of your memory quota ({:.2}GB/{} GB)",
            quota.memory_percentage as i32,
            quota.current_memory_mb as f64 / 1024.0,
            quota.max_memory_gb
        ));
    }

    let response = QuotaResponse { quota, warnings };

    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuotaRequest {
    pub max_instances: Option<i32>,
    pub max_memory_gb: Option<i32>,
    pub max_api_keys: Option<i32>,
}

/// Update quota limits for an organization (admin only)
pub async fn update_quota(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<UpdateQuotaRequest>,
) -> Result<Json<ApiResponse<QuotaInfo>>, ErrorResponse> {
    // Verify user is owner/admin of organization
    let membership = sqlx::query!(
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
    })?;

    let Some(membership) = membership else {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Access denied to this organization".to_string())),
        ));
    };

    // Only owners and admins can update quotas
    if membership.role != "owner" && membership.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                "Only organization owners and admins can update quotas".to_string(),
            )),
        ));
    }

    // Validate values
    if let Some(val) = payload.max_instances {
        if val < 1 || val > 1000 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("max_instances must be between 1 and 1000".to_string())),
            ));
        }
    }

    if let Some(val) = payload.max_memory_gb {
        if val < 1 || val > 10000 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("max_memory_gb must be between 1 and 10000".to_string())),
            ));
        }
    }

    if let Some(val) = payload.max_api_keys {
        if val < 1 || val > 1000 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("max_api_keys must be between 1 and 1000".to_string())),
            ));
        }
    }

    // Update quotas
    let quota_service = QuotaService::new(Arc::new(state.db_pool.clone()));
    quota_service
        .update_quota_limits(
            org_id,
            payload.max_instances,
            payload.max_memory_gb,
            payload.max_api_keys,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to update quotas: {}", e))),
            )
        })?;

    // Return updated quota info
    let quota = quota_service.get_quota_info(org_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("{}", e))),
        )
    })?;

    Ok(Json(ApiResponse::success(quota)))
}
