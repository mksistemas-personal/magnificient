-- Create enumeration for entity types that can be linked to an organization
DO
$$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_type t
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'organization_entity_type'
          AND n.nspname = current_schema()
    ) THEN
        CREATE TYPE organization_entity_type AS ENUM ('CLIENT', 'PRODUCER');
    END IF;
END
$$;

-- Table linking an organization with related entities (clients, producers, etc.)
CREATE TABLE IF NOT EXISTS organization_links
(
    id              BIGINT PRIMARY KEY,
    organization_id BIGINT                   NOT NULL,
    entity_id       BIGINT                   NOT NULL,
    entity_type     organization_entity_type NOT NULL,
    linked_at       TIMESTAMPTZ              NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at      TIMESTAMPTZ              NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by      VARCHAR(100)             NOT NULL DEFAULT 'system',
    CONSTRAINT fk_organization_links_organization
        FOREIGN KEY (organization_id) REFERENCES organizations (id)
            ON DELETE CASCADE,
    CONSTRAINT uq_organization_links_entity
        UNIQUE (organization_id, entity_type, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_organization_links_org_id ON organization_links (organization_id);
CREATE INDEX IF NOT EXISTS idx_organization_links_entity ON organization_links (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_organization_links_linked_at ON organization_links (linked_at);

COMMENT ON TABLE organization_links IS 'Links organizations to other domain entities (e.g. client, producer)';
COMMENT ON COLUMN organization_links.id IS 'Unique TSID (64-bit integer)';
COMMENT ON COLUMN organization_links.organization_id IS 'ID of the organization being linked';
COMMENT ON COLUMN organization_links.entity_id IS 'ID of the target entity linked to the organization';
COMMENT ON COLUMN organization_links.entity_type IS 'Enumeration of target entity type (CLIENT, PRODUCER)';
COMMENT ON COLUMN organization_links.linked_at IS 'Timestamp when the link was established';
COMMENT ON COLUMN organization_links.created_at IS 'Creation timestamp in UTC';
COMMENT ON COLUMN organization_links.created_by IS 'Creator identifier';
