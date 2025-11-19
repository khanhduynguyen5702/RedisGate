-- Add quota columns to organizations table
ALTER TABLE organizations
ADD COLUMN IF NOT EXISTS max_redis_instances INTEGER DEFAULT 5 NOT NULL,
ADD COLUMN IF NOT EXISTS max_memory_gb INTEGER DEFAULT 10 NOT NULL,
ADD COLUMN IF NOT EXISTS max_api_keys INTEGER DEFAULT 10 NOT NULL;

-- Create instance quotas tracking table
CREATE TABLE IF NOT EXISTS instance_quotas (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    current_instances INTEGER DEFAULT 0 NOT NULL,
    current_memory_mb INTEGER DEFAULT 0 NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Function to update quota when instance is created or deleted
CREATE OR REPLACE FUNCTION update_instance_quota()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.deleted_at IS NULL THEN
        -- Instance created
        INSERT INTO instance_quotas (organization_id, current_instances, current_memory_mb)
        VALUES (NEW.organization_id, 1, COALESCE(NEW.max_memory, 256))
        ON CONFLICT (organization_id) DO UPDATE
        SET current_instances = instance_quotas.current_instances + 1,
            current_memory_mb = instance_quotas.current_memory_mb + COALESCE(NEW.max_memory, 256),
            updated_at = NOW();
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
            -- Instance soft deleted
            UPDATE instance_quotas
            SET current_instances = GREATEST(0, current_instances - 1),
                current_memory_mb = GREATEST(0, current_memory_mb - COALESCE(OLD.max_memory, 256)),
                updated_at = NOW()
            WHERE organization_id = OLD.organization_id;
        ELSIF OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN
            -- Instance restored
            INSERT INTO instance_quotas (organization_id, current_instances, current_memory_mb)
            VALUES (NEW.organization_id, 1, COALESCE(NEW.max_memory, 256))
            ON CONFLICT (organization_id) DO UPDATE
            SET current_instances = instance_quotas.current_instances + 1,
                current_memory_mb = instance_quotas.current_memory_mb + COALESCE(NEW.max_memory, 256),
                updated_at = NOW();
        END IF;
    ELSIF TG_OP = 'DELETE' AND OLD.deleted_at IS NULL THEN
        -- Instance hard deleted
        UPDATE instance_quotas
        SET current_instances = GREATEST(0, current_instances - 1),
            current_memory_mb = GREATEST(0, current_memory_mb - COALESCE(OLD.max_memory, 256)),
            updated_at = NOW()
        WHERE organization_id = OLD.organization_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for redis_instances table
DROP TRIGGER IF EXISTS redis_instance_quota_trigger ON redis_instances;
CREATE TRIGGER redis_instance_quota_trigger
AFTER INSERT OR UPDATE OR DELETE ON redis_instances
FOR EACH ROW EXECUTE FUNCTION update_instance_quota();

-- Initialize quota data for existing organizations
INSERT INTO instance_quotas (organization_id, current_instances, current_memory_mb)
SELECT
    o.id,
    COUNT(ri.id)::INTEGER,
    COALESCE(SUM(ri.max_memory), 0)::INTEGER
FROM organizations o
LEFT JOIN redis_instances ri ON o.id = ri.organization_id AND ri.deleted_at IS NULL
GROUP BY o.id
ON CONFLICT (organization_id) DO UPDATE
SET current_instances = EXCLUDED.current_instances,
    current_memory_mb = EXCLUDED.current_memory_mb,
    updated_at = NOW();

-- Create index for better performance
CREATE INDEX IF NOT EXISTS idx_instance_quotas_org_id ON instance_quotas(organization_id);

