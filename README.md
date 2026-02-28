# observistral 🐈‍⬛

> **AI-powered observability using Mistral AI** — built for Mistral AI's worldwide hackathon.

`observistral` is a command-line tool that ingests your logs or metrics and uses
the [Mistral AI](https://mistral.ai/) chat API to surface errors, anomalies,
root causes, and remediation steps in plain language.

---

## Features

- 📄 **Stdin or file input** — pipe logs directly or point to a file
- 🤖 **Mistral AI analysis** — powered by `mistral-large-latest` (configurable)
- 🔍 **Structured insights** — summary, anomalies, root causes, and remediation
- ✏️  **Custom prompts** — override the default analysis with your own question

---

## Requirements

- Rust 1.75+ (uses the 2024 edition)
- A [Mistral AI API key](https://console.mistral.ai/)

---

## Installation

```bash
git clone https://github.com/RyoSpiralArchitect/observistral.git
cd observistral
cargo build --release
# binary is at ./target/release/observistral
```

---

## Usage

### Set your API key

```bash
export MISTRAL_API_KEY="your_key_here"
```

### Analyze logs from stdin

```bash
journalctl -n 200 | observistral analyze
```

### Analyze a log file

```bash
observistral analyze --file /var/log/app/error.log
```

### Use a specific model

```bash
observistral analyze --file app.log --model mistral-small-latest
```

### Ask a custom question about the logs

```bash
observistral analyze --file app.log --prompt "Are there any authentication failures?"
```

---

## Example output

```
🔍 Analyzing with mistral-large-latest…

**Summary**
The application experienced a spike of 5xx errors between 14:32 and 14:47 UTC,
coinciding with a database connection timeout storm.

**Anomalies detected**
- 312 occurrences of `connection refused` to postgres:5432 in a 15-minute window
- Memory usage climbed from 42 % to 94 % before the OOM killer fired

**Potential root causes**
1. Connection pool exhausted due to long-running queries (several >30 s entries)
2. Missing index on `events.created_at` causing full table scans under load

**Recommended remediation**
- Add index: `CREATE INDEX CONCURRENTLY ON events (created_at);`
- Increase `max_connections` in `pg_hba.conf` or tune the connection pool size
- Add a query timeout of 10 s to prevent pool starvation
```

---

## Project structure

```
observistral/
├── Cargo.toml
└── src/
    └── main.rs   # CLI + Mistral API integration
```

---

## License

MIT
