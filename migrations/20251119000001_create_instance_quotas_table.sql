-- Create instance_quotas table for tracking organization resource usage
-- This table is used by the QuotaService to efficiently check limits

CREATE TABLE IF NOT EXISTS instance_quotas (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    current_instances INTEGER NOT NULL DEFAULT 0,
    current_memory_mb INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_instance_quotas_org_id ON instance_quotas(organization_id);

-- Create function to update quota automatically
CREATE OR REPLACE FUNCTION update_instance_quota()
RETURNS TRIGGER AS $$
BEGIN
    -- Update quota counters when instances are created/deleted
    INSERT INTO instance_quotas (organization_id, current_instances, current_memory_mb, updated_at)
    SELECT
        organization_id,
        COUNT(*)::INTEGER,
        COALESCE(SUM((max_memory / 1024 / 1024)::INTEGER), 0)::INTEGER,
        NOW()
    FROM redis_instances
    WHERE organization_id = COALESCE(NEW.organization_id, OLD.organization_id)
      AND deleted_at IS NULL
    GROUP BY organization_id
    ON CONFLICT (organization_id)
    DO UPDATE SET
        current_instances = EXCLUDED.current_instances,
        current_memory_mb = EXCLUDED.current_memory_mb,
        updated_at = NOW();

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update quotas
DROP TRIGGER IF EXISTS trigger_update_instance_quota ON redis_instances;
CREATE TRIGGER trigger_update_instance_quota
    AFTER INSERT OR UPDATE OR DELETE ON redis_instances
    FOR EACH ROW
    EXECUTE FUNCTION update_instance_quota();

-- Initialize quotas for existing organizations
INSERT INTO instance_quotas (organization_id, current_instances, current_memory_mb, updated_at)
SELECT
    o.id,
    COALESCE(COUNT(ri.id), 0)::INTEGER,
    COALESCE(SUM((ri.max_memory / 1024 / 1024)::INTEGER), 0)::INTEGER,
    NOW()
FROM organizations o
LEFT JOIN redis_instances ri ON o.id = ri.organization_id AND ri.deleted_at IS NULL
GROUP BY o.id
ON CONFLICT (organization_id) DO NOTHING;

