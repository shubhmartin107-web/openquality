CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT,
    role TEXT NOT NULL DEFAULT 'viewer'
        CHECK (role IN ('admin', 'editor', 'viewer', 'member')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    key_hash TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS data_sources (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    source_type TEXT NOT NULL,
    config_json JSONB NOT NULL DEFAULT '{}',
    connection_status TEXT NOT NULL DEFAULT 'unknown',
    last_tested_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS expectation_suites (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    description TEXT,
    data_source_id UUID REFERENCES data_sources(id),
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS expectations (
    id UUID PRIMARY KEY,
    suite_id UUID NOT NULL REFERENCES expectation_suites(id) ON DELETE CASCADE,
    expectation_type TEXT NOT NULL,
    column_name TEXT,
    kwargs_json JSONB NOT NULL DEFAULT '{}',
    tolerance DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    position INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS suite_runs (
    id UUID PRIMARY KEY,
    suite_id UUID NOT NULL REFERENCES expectation_suites(id),
    data_source_id UUID REFERENCES data_sources(id),
    status TEXT NOT NULL DEFAULT 'pending',
    results_json JSONB,
    summary_json JSONB,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS monitors (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    monitor_type TEXT NOT NULL,
    table_name TEXT NOT NULL,
    data_source_id UUID REFERENCES data_sources(id),
    config_json JSONB NOT NULL DEFAULT '{}',
    schedule_cron TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    auto_threshold BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS monitor_history (
    id BIGSERIAL PRIMARY KEY,
    monitor_id UUID NOT NULL REFERENCES monitors(id),
    score DOUBLE PRECISION NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_monitor_history_mid_time
    ON monitor_history(monitor_id, checked_at DESC);

CREATE TABLE IF NOT EXISTS incidents (
    id UUID PRIMARY KEY,
    workspace_id UUID REFERENCES workspaces(id),
    monitor_id TEXT NOT NULL,
    suite_run_id UUID,
    severity TEXT NOT NULL DEFAULT 'warning',
    status TEXT NOT NULL DEFAULT 'open',
    message TEXT NOT NULL,
    group_key TEXT,
    root_cause_hints_json JSONB,
    detail_json JSONB,
    owner_id UUID REFERENCES users(id),
    escalation_level INT NOT NULL DEFAULT 0,
    snoozed_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS column_profiles (
    id BIGSERIAL PRIMARY KEY,
    data_source_id UUID NOT NULL REFERENCES data_sources(id),
    table_name TEXT NOT NULL,
    column_name TEXT NOT NULL,
    row_count BIGINT NOT NULL DEFAULT 0,
    null_count BIGINT NOT NULL DEFAULT 0,
    distinct_count BIGINT NOT NULL DEFAULT 0,
    min_val DOUBLE PRECISION,
    max_val DOUBLE PRECISION,
    mean_val DOUBLE PRECISION,
    stddev_val DOUBLE PRECISION,
    quantiles_json JSONB,
    histogram_json JSONB,
    top_k_values_json JSONB,
    profiled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_column_profiles_lookup
    ON column_profiles(data_source_id, table_name, column_name, profiled_at DESC);

CREATE TABLE IF NOT EXISTS lineage_edges (
    id BIGSERIAL PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    source_table TEXT NOT NULL,
    source_column TEXT,
    target_table TEXT NOT NULL,
    target_column TEXT,
    transformation TEXT,
    confidence DOUBLE PRECISION DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_workspace_id ON users(workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_data_sources_workspace_id ON data_sources(workspace_id);
CREATE INDEX IF NOT EXISTS idx_expectation_suites_workspace_id ON expectation_suites(workspace_id);
CREATE INDEX IF NOT EXISTS idx_expectation_suites_data_source_id ON expectation_suites(data_source_id);
CREATE INDEX IF NOT EXISTS idx_expectations_suite_id ON expectations(suite_id);
CREATE INDEX IF NOT EXISTS idx_suite_runs_suite_id ON suite_runs(suite_id);
CREATE INDEX IF NOT EXISTS idx_suite_runs_data_source_id ON suite_runs(data_source_id);
CREATE INDEX IF NOT EXISTS idx_monitors_workspace_id ON monitors(workspace_id);
CREATE INDEX IF NOT EXISTS idx_monitors_data_source_id ON monitors(data_source_id);
CREATE INDEX IF NOT EXISTS idx_incidents_workspace_id ON incidents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_incidents_owner_id ON incidents(owner_id);
CREATE INDEX IF NOT EXISTS idx_incidents_resolved_by ON incidents(resolved_by);
CREATE INDEX IF NOT EXISTS idx_lineage_edges_workspace_id ON lineage_edges(workspace_id);

CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    workspace_id UUID REFERENCES workspaces(id),
    user_id UUID REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    details_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audit_log_ws_time
    ON audit_log(workspace_id, created_at DESC);
