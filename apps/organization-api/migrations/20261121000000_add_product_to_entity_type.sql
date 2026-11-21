-- Add 'PRODUCT' to the organization_entity_type enum

DO
$$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'organization_entity_type'
          AND n.nspname = current_schema()
          AND e.enumlabel = 'PRODUCT'
    ) THEN
        ALTER TYPE organization_entity_type ADD VALUE 'PRODUCT';
    END IF;
END
$$;

-- Update the column comment to reflect the new values
COMMENT ON COLUMN organization_links.entity_type IS 'Enumeration of target entity type (CLIENT, SUPPLIER, PRODUCT)';
