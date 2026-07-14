# OpenQuality V2

**Enterprise-grade data quality + observability platform.**

Built on a Rust core with PostgreSQL persistence, 10 monitor types, 17 expectation types,
alert routing (Slack/PagerDuty/Webhook), root-cause analysis with causal inference,
a REST API with 43+ endpoints, a React dashboard, and integrations with dbt, Airflow,
Great Expectations, and SQL lineage.

## Architecture

```
openquality/
├── crates/
│   ├── openquality-core/        # Core types, 10 monitors, 17 expectations, stats, alerts, root cause
│   ├── openquality-store/       # PgStore (PostgreSQL via sqlx) + InMemoryStore
│   ├── openquality-auth/        # JWT auth, Argon2 passwords, API keys, RBAC (5 roles)
│   ├── openquality-server/      # Axum REST API (43+ endpoints, CORS, SPA serving)
│   ├── openquality-cli/         # CLI with 6 command groups (health, workspaces, monitors, incidents, data-sources, integrations)
│   ├── openquality-scheduler/   # Cron-based monitor scheduler (30s interval)
│   ├── openquality-mcp/         # MCP server for LLM integration (8 tools)
│   ├── openquality-connections/ # Postgres/Snowflake/BigQuery connectors
│   ├── openquality-integrations/# dbt, Airflow, Great Expectations, SQL lineage parsers
│   └── openquality-python/      # PyO3 bindings (KS test, JS divergence, z-score)
├── web/                         # React + Vite + TypeScript + Tailwind dashboard
├── python/                      # Python reference implementation
├── demo/                        # Drift detection + broken pipeline demos
├── benchmarks/                  # Injected anomaly detection accuracy
└── docs/                        # OpenAPI spec
```

## Quick Start

```bash
# Build everything
cargo build --workspace

# Run all 114+ Rust tests
cargo test --workspace

# Start the server (auto-detects PgStore if DATABASE_URL is set)
cargo run -p openquality-server

# Build and serve the web UI
cd web && npm install && npm run build
# The server serves web/dist/ at http://localhost:8080

# CLI usage
cargo run -p openquality-cli -- health
cargo run -p openquality-cli -- workspaces list
OQ_API_TOKEN=<token> cargo run -p openquality-cli -- incidents list
```

## Monitors (10 types)

| Monitor | Detection Method |
|---------|-----------------|
| Freshness | Stale data with auto-thresholds (MAD) |
| Volume | Row count anomalies (MAD/IQR/3-sigma) |
| Schema | Column added/removed/type change |
| Distribution | KS test, JS divergence, chi-squared |
| Correlation | Pearson/Spearman correlation between columns |
| Uniqueness | Duplicate row ratio vs threshold |
| Referential Integrity | Orphan row count between source and target |
| Custom SQL | Observed vs expected with comparison operators |
| ML Drift | Prediction/feature/accuracy drift scoring |
| Cost | Query/storage/compute cost vs budget |

## Expectations (17 types)

`NotNull`, `Unique`, `Between`, `MatchRegex`, `NotMatchRegex`,
`DistinctValuesEqualSet`, `DistinctValuesContainedInSet`, `RowCountBetween`,
`ColumnMeanBetween`, `ColumnStddevBetween`, `ColumnMinBetween`, `ColumnMaxBetween`,
`ColumnValuesToBeInSet`, `ColumnKLDivergenceLessThan`, `ColumnQuantileBetween`,
`TableColumnsMatchOrderedList`, `Custom`

## Integrations

- **dbt** — Parse `manifest.json` → models, sources, column metadata, lineage edges
- **Airflow** — Webhook receiver for DAG run / task instance events
- **Great Expectations** — Translate GE suites → OpenQuality expectations (16+ mappings)
- **Lineage** — SQL parser for `CREATE TABLE AS SELECT` / `INSERT INTO` → column-level lineage

## API

Full REST API at `/api/v1/`:

```
Health    GET  /health
Auth      POST /api/v1/auth/login|register|refresh
          GET|POST /api/v1/auth/api-keys
Workspaces GET|POST /api/v1/workspaces
Monitors  GET|POST /api/v1/workspaces/{ws}/monitors
          GET|PUT|DELETE /api/v1/monitors/{id}
          POST /api/v1/monitors/{id}/run|history
Incidents GET /api/v1/incidents
          GET|POST /api/v1/incidents/{id}/acknowledge|resolve|snooze
Data Sources GET|POST /api/v1/workspaces/{ws}/data-sources
Integrations POST /api/v1/integrations/dbt/parse-manifest
             POST /api/v1/integrations/airflow/webhook
             POST /api/v1/integrations/ge/translate
             POST /api/v1/integrations/lineage/parse-sql
```

See `docs/openapi.json` for the full spec.

## Root Cause Analysis V2

Causal inference engine that computes:

- **Granger causality** — F-statistic whether past values predict current anomalies
- **PCA contribution** — Variance-based anomaly dimension isolation
- **Dimension isolation** — Which column contributed most to the drift
- **Deployment correlation** — Timeline overlap with known deployments

## Alert Channels

- **Stdout** — Console output with formatting
- **JSON** — Structured JSON output
- **Slack** — Webhook POST with formatted message
- **PagerDuty** — Events API v2 trigger
- **Webhook** — Generic HTTP POST of the full incident

## Storage

- **InMemoryStore** — HashMap-backed, for tests (default)
- **PgStore** — PostgreSQL via sqlx with 12 tables, auto-selected when `DATABASE_URL` is set

## Authentication

- JWT Bearer tokens (HS256) with automatic refresh
- Argon2 password hashing
- API key authentication (SHA-256 hashed)
- RBAC: Owner → Admin → Editor → Member → Viewer

## License

Apache 2.0
