use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub organization_id: Uuid,
    pub max_instances: i32,
    pub current_instances: i32,
    pub max_memory_gb: i32,
    pub current_memory_mb: i32,
    pub available_memory_mb: i32,
    pub max_api_keys: i32,
    pub current_api_keys: i32,
    pub instances_percentage: f64,
    pub memory_percentage: f64,
}

#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("Maximum instances reached: {current}/{max}. Please upgrade your plan or delete unused instances.")]
    MaxInstancesReached { current: i32, max: i32 },

    #[error("Memory limit exceeded: requested {requested}MB, available {available}MB ({total_gb}GB total)")]
    MemoryLimitExceeded { requested: i32, available: i32, total_gb: i32 },

    #[error("Maximum API keys reached: {current}/{max}")]
    MaxApiKeysReached { current: i32, max: i32 },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid organization ID: {0}")]
    InvalidOrgId(#[from] uuid::Error),

    #[error("Organization not found")]
    OrgNotFound,
}

pub struct QuotaService {
    db: Arc<PgPool>,
}

impl QuotaService {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Check if organization can create a new Redis instance
    pub async fn check_can_create_instance(
        &self,
        org_id: Uuid,
        memory_mb: i32,
    ) -> Result<(), QuotaError> {
        let quota = self.get_quota_info(org_id).await?;

        // Check instance count limit
        if quota.current_instances >= quota.max_instances {
            return Err(QuotaError::MaxInstancesReached {
                current: quota.current_instances,
                max: quota.max_instances,
            });
        }

        // Check memory limit
        let total_memory_mb = quota.current_memory_mb + memory_mb;
        let max_memory_mb = quota.max_memory_gb * 1024;

        if total_memory_mb > max_memory_mb {
            return Err(QuotaError::MemoryLimitExceeded {
                requested: memory_mb,
                available: quota.available_memory_mb,
                total_gb: quota.max_memory_gb,
            });
        }

        Ok(())
    }

    /// Check if organization can create a new API key
    pub async fn check_can_create_api_key(&self, org_id: Uuid) -> Result<(), QuotaError> {
        let result = sqlx::query!(
            r#"
            SELECT
                o.max_api_keys,
                COUNT(ak.id)::INTEGER as current_api_keys
            FROM organizations o
            LEFT JOIN api_keys ak ON o.id = ak.organization_id AND ak.is_active = true
            WHERE o.id = $1
            GROUP BY o.id, o.max_api_keys
            "#,
            org_id
        )
        .fetch_optional(&*self.db)
        .await?;

        let Some(record) = result else {
            return Err(QuotaError::OrgNotFound);
        };

        let current = record.current_api_keys.unwrap_or(0);
        let max = record.max_api_keys.unwrap_or(10);

        if current >= max {
            return Err(QuotaError::MaxApiKeysReached {
                current,
                max,
            });
        }

        Ok(())
    }

    /// Get current quota information for an organization
    pub async fn get_quota_info(&self, org_id: Uuid) -> Result<QuotaInfo, QuotaError> {
        let result = sqlx::query!(
            r#"
            SELECT
                o.id as organization_id,
                o.max_redis_instances,
                o.max_memory_gb,
                o.max_api_keys,
                COALESCE(q.current_instances, 0)::INTEGER as current_instances,
                COALESCE(q.current_memory_mb, 0)::INTEGER as current_memory_mb,
                COUNT(DISTINCT ak.id)::INTEGER as current_api_keys
            FROM organizations o
            LEFT JOIN instance_quotas q ON o.id = q.organization_id
            LEFT JOIN api_keys ak ON o.id = ak.organization_id AND ak.is_active = true
            WHERE o.id = $1 AND o.is_active = true
            GROUP BY o.id, o.max_redis_instances, o.max_memory_gb, o.max_api_keys,
                     q.current_instances, q.current_memory_mb
            "#,
            org_id
        )
        .fetch_optional(&*self.db)
        .await?;

        let Some(record) = result else {
            return Err(QuotaError::OrgNotFound);
        };

        let current_instances = record.current_instances.unwrap_or(0);
        let current_memory_mb = record.current_memory_mb.unwrap_or(0);
        let max_memory_mb = record.max_memory_gb * 1024;
        let available_memory_mb = max_memory_mb - current_memory_mb;

        let max_redis_instances = record.max_redis_instances.unwrap_or(5);
        let instances_percentage = if max_redis_instances > 0 {
            (current_instances as f64 / max_redis_instances as f64) * 100.0
        } else {
            0.0
        };

        let memory_percentage = if max_memory_mb > 0 {
            (current_memory_mb as f64 / max_memory_mb as f64) * 100.0
        } else {
            0.0
        };

        Ok(QuotaInfo {
            organization_id: record.organization_id,
            max_instances: max_redis_instances,
            current_instances,
            max_memory_gb: record.max_memory_gb,
            current_memory_mb,
            available_memory_mb,
            max_api_keys: record.max_api_keys.unwrap_or(10),
            current_api_keys: record.current_api_keys.unwrap_or(0),
            instances_percentage,
            memory_percentage,
        })
    }

    /// Update organization quota limits (admin only)
    pub async fn update_quota_limits(
        &self,
        org_id: Uuid,
        max_instances: Option<i32>,
        max_memory_gb: Option<i32>,
        max_api_keys: Option<i32>,
    ) -> Result<(), QuotaError> {
        let mut query = String::from("UPDATE organizations SET ");
        let mut updates = Vec::new();
        let mut param_count = 1;

        if max_instances.is_some() {
            updates.push(format!("max_redis_instances = ${}", param_count));
            param_count += 1;
        }
        if max_memory_gb.is_some() {
            updates.push(format!("max_memory_gb = ${}", param_count));
            param_count += 1;
        }
        if max_api_keys.is_some() {
            updates.push(format!("max_api_keys = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Ok(());
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(", updated_at = NOW() WHERE id = ${}", param_count));

        let mut query_builder = sqlx::query(&query);

        if let Some(val) = max_instances {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = max_memory_gb {
            query_builder = query_builder.bind(val);
        }
        if let Some(val) = max_api_keys {
            query_builder = query_builder.bind(val);
        }

        query_builder = query_builder.bind(org_id);

        query_builder.execute(&*self.db).await?;

        Ok(())
    }
}

