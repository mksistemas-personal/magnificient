CREATE TABLE IF NOT EXISTS public.tenants (
    slug VARCHAR(63) PRIMARY KEY,
    schema_name VARCHAR(63) NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT tenants_slug_format CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    CONSTRAINT tenants_schema_format CHECK (schema_name ~ '^tenant_[a-z0-9]+(_[a-z0-9]+)*$')
);
