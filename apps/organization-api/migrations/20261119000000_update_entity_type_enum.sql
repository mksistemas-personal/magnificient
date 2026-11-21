-- Replace 'PRODUCER' with 'SUPPLIER' in the organization_entity_type enum
-- This migration renames the existing 'PRODUCER' value to 'SUPPLIER'

DO
$$
BEGIN
    -- Rename the existing 'PRODUCER' value to 'SUPPLIER' if 'PRODUCER' exists
    IF EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'organization_entity_type'
          AND n.nspname = current_schema()
          AND e.enumlabel = 'PRODUCER'
    ) THEN
        ALTER TYPE organization_entity_type RENAME VALUE 'PRODUCER' TO 'SUPPLIER';
    END IF;
END
$$;

-- Update the column comment to reflect the new values
COMMENT ON COLUMN organization_links.entity_type IS 'Enumeration of target entity type (CLIENT, SUPPLIER)';
