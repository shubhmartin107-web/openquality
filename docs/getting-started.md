# Getting Started with OpenQuality

## Installation

```bash
# Clone the repo
git clone <repo> && cd openquality

# Build Rust core
cargo build --workspace

# Install Python deps
pip install pandas numpy
```

## Defining an Expectation Suite

```python
from openquality.expectations.core import (
    ExpectationSuite, ExpectationRunner,
    expect_not_null, expect_unique, expect_between
)
import pandas as pd

suite = ExpectationSuite("orders")
suite.add(expect_not_null("order_id"))
suite.add(expect_unique("order_id"))
suite.add(expect_between("amount", 0, 10000))

df = pd.read_csv("orders.csv")
result = ExpectationRunner.run(suite, df)
print(result.summary())
```

## Setting Up a Freshness Monitor

```python
from openquality.monitors.core import MonitorConfig, FreshnessMonitor
from datetime import datetime, timezone, timedelta

config = MonitorConfig.freshness("orders-freshness", "orders", max_age_seconds=3600)
monitor = FreshnessMonitor(config)

last_updated = datetime.now(timezone.utc) - timedelta(hours=2)
result = monitor.check(last_updated)

if result.alert:
    print(f"ALERT: {result.message}")
```

## Setting Up a Volume Monitor with Auto-Threshold

```python
from openquality.monitors.core import MonitorConfig, VolumeMonitor

config = MonitorConfig.volume("orders-volume", "orders", sensitivity=3.0)
monitor = VolumeMonitor(config)

# Feed historical data
for count in [9500, 10200, 9800, 10500, 9900]:
    monitor.check(count)

# Check current
result = monitor.check(1000)
print(f"Alert: {result.alert}, Score: {result.score}, Threshold: {result.threshold}")
```

## Running the CLI

```bash
cargo run -p openquality-cli -- validate --file data.csv --expectation not_null --column id
cargo run -p openquality-cli -- health
```

## MCP Server

Configure in `opencode.json`:

```json
{
  "mcp": {
    "openquality": {
      "type": "local",
      "command": ["cargo", "run", "-p", "openquality-mcp"],
      "enabled": true
    }
  }
}
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `openquality_run_suite` | Run an expectation suite |
| `openquality_validate_column` | Validate a single column |
| `openquality_list_monitors` | List registered monitors |
| `openquality_run_monitor` | Execute a monitor manually |
| `openquality_list_incidents` | Query incidents |
| `openquality_resolve_incident` | Resolve an incident |
| `openquality_root_cause` | Generate root-cause hints |
| `openquality_health` | Health check |
