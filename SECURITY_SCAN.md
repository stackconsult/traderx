# TraderX Security Scan Report

Date: 2026-04-16
Scanner: manual review + ripgrep pattern search
Scope: stackconsult/traderx @ main

## Summary

| Severity  | Count |
|-----------|-------|
| Critical  | 4     |
| High      | 4     |
| Medium    | 4     |
| Low/Info  | 3     |

The same hardcoded GitHub Personal Access Token appears in 6 files across
scripts, docs, and `.windsurf/` workflows. The trading engine's control-plane
HTTP API (`START`, `STOP`, `FLATTEN`, `/api/config`, `/api/strategy`) is
exposed on `0.0.0.0:3000` with no authentication and `CorsLayer::permissive()`.
The model-serving HTTP API has the same permissive CORS. Grafana in
`docker-compose.yml` ships with the default `admin`/`admin` credential.

This PR fixes the four critical issues listed below and records the remaining
issues for follow-up.

---

## Critical

### C1. Hardcoded GitHub Personal Access Token committed to the repo
**Files**
- `scripts/check_pr_status.py:8`
- `scripts/autonomous_audit_loop.py:15`
- `scripts/check_actions.py:10` (embedded as `os.environ.get(..., "<pat>")` default)
- `.github/mcp_github_setup.md:21-22`
- `.windsurf/workflows/github-mcp-integration.md:30`
- `VALIDATION_STATUS.md:12` (prefix only, but still leaks)

A fine-grained PAT with prefix `github_pat_11BZU7ESI0Bf16uUG09PQq_...` is
committed in plaintext. Anyone with read access to the repo history gets a
token that can act against the `stackconsult` org.

**Fix (this PR):** Replace all occurrences with reads from
`os.environ["GITHUB_TOKEN"]` (or the equivalent placeholder in markdown) and
fail fast if unset. **The token itself must still be rotated out-of-band** —
removing it from `HEAD` does not remove it from git history.

**Owner action required:**
1. Revoke the PAT at https://github.com/settings/tokens
2. Issue a new token, store it in the session/org secret `GITHUB_TOKEN`
3. Consider running `git filter-repo` / GitHub's secret scanning push
   protection to purge it from history

### C2. Trading engine control API has no authentication + permissive CORS
**File:** `packages/hft-system/apps/trading_engine/src/server.rs`

The HFT trading engine binds `0.0.0.0:3000` and exposes:
- `POST /api/control` — `START` / `STOP` / `FLATTEN` the live engine
- `POST /api/config` — change max loss / target profit
- `POST /api/strategy` — switch the active strategy
- `GET/DELETE /api/history` — wipe trade history
- `GET /api/pnl_series`, `/api/logs`

There is no authentication middleware, and `CorsLayer::permissive()` allows
any origin with any method. A user visiting a malicious page on the same
network (or over the internet, if the port is reachable) can `FLATTEN` the
portfolio or flip the strategy.

**Fix (this PR):** Replace `CorsLayer::permissive()` with an explicit
localhost-only allow-list using `AllowOrigin::list(...)`, and bind to
`127.0.0.1` by default (controllable via `TRADERX_BIND_ADDR`). Authentication
is called out as a follow-up (see "Remaining work").

### C3. Model-serving inference API uses `CorsLayer::permissive()`
**File:** `packages/model-serving/src/server.rs:75`

Same class of issue as C2: `/models/*/predict`, `/models/*/ab-test`, and
`POST /models` (model registration) are exposed with wildcard CORS and no
auth. In a browser context this allows cross-site inference requests that
will be billed/attributed to the caller.

**Fix (this PR):** Replace with an explicit origin list driven by env var,
defaulting to closed.

### C4. Grafana ships with default `admin` / `admin` credentials
**File:** `docker-compose.yml:160`

```
GF_SECURITY_ADMIN_PASSWORD=admin
```

Grafana is exposed on `:3001`. An attacker reaching that port gets full admin
access with the default credential.

**Fix (this PR):** Require `GRAFANA_ADMIN_PASSWORD` via `.env` (same
`:?VAR must be set` syntax already used for `POSTGRES_PASSWORD`).

---

## High

### H1. API Gateway CORS allows wildcard methods & headers with credentials
**File:** `packages/api-gateway/src/main.py:53-59`

```python
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "https://traderx.com"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

The origin list is scoped, which is good. `allow_methods=["*"]` and
`allow_headers=["*"]` combined with `allow_credentials=True` is discouraged
by the Fetch spec — browsers will accept it, and it widens the attack
surface for any future XSS/origin-confusion bug. Origins should also be
configurable via env for non-prod deployments.

**Fix (this PR):** Narrow to the methods and headers actually used
(`GET, POST, PUT, DELETE, OPTIONS` / `Authorization, Content-Type,
X-Tenant-ID, X-Request-ID`) and source allowed origins from
`CORS_ALLOWED_ORIGINS`.

### H2. API Gateway endpoints have no authentication
**File:** `packages/api-gateway/src/main.py`

Every route (`/api/v1/status`, `/api/v1/strategies`, `/api/v1/handoff`,
`/api/v1/handoff/{id}`, `/ws`) is unauthenticated. The rate-limiter even
acknowledges it: `_extract_tenant_id` has a `TODO: Decode JWT and extract
tenant_id` and falls back to `"anonymous"` for any caller, so the
"per-tenant" rate limiting is effectively shared across all unauthenticated
traffic.

**Not fixed in this PR** — requires choosing/wiring an auth scheme (the
repo already depends on `python-jose[cryptography]` and `passlib[bcrypt]`).
Tracked as follow-up.

### H3. OMS observability server opt-in CORS uses `*`
**File:** `packages/oms-engine/src/observability_server.rs:111-115`

When `enable_cors=true`, origin is hardcoded to `*`. The default is `false`
so this is only exploitable if an operator enables it.

**Fix (this PR):** Drive the origin list from config (same `ObsServerConfig`)
and default to `None`.

### H4. `pickle.loads` on network-sourced buffers
**File:** `packages/sugaformer/models/lavis/util/misc.py:146`

```python
data_list.append(pickle.loads(buffer))
```

Vendored from LAVIS; used in distributed all_gather. If any peer is
untrusted, this is RCE. Low immediate exploitability in a single-tenant
cluster, but worth tracking.

**Not fixed in this PR** — vendored ML code, changes would diverge from
upstream. Recommend isolating the distributed ranks on a private network
(already the case in `docker-compose.yml`).

---

## Medium

### M1. Dynamic SQL with interpolated table names
**File:** `packages/database/src/tenant_manager.py:202, 224`

```python
count = await conn.fetchval(f"SELECT COUNT(*) FROM {table}")
await conn.execute(f"UPDATE {table} SET tenant_id = $1 WHERE tenant_id IS NULL", tenant_id)
```

The `table` variable comes from a hardcoded in-function list, so there is no
immediate injection path. It is still a code smell — if that list is ever
made configurable, this becomes a vulnerability. Recommend validating
against an allow-list at call time or quoting via `asyncpg`'s identifier
quoting helpers.

**Not fixed in this PR** (no exploitable path today). Tracked as follow-up.

### M2. `subprocess.run(..., shell=True)` on command strings
**Files:**
- `scripts/github_sync_check.py:20`
- `scripts/self_healing_guard.py:33`

Both pass constant strings today, so not directly exploitable, but the
pattern is fragile. Prefer `shell=False` with a list-form `cmd`.

**Not fixed in this PR** — out-of-scope helper scripts.

### M3. FastAPI `/docs` and `/redoc` always enabled
**File:** `packages/api-gateway/src/main.py:44-50`

`docs_url="/docs"` and `redoc_url="/redoc"` are hardcoded. In production
the OpenAPI schema is an information-disclosure aid for attackers.

**Not fixed in this PR** — low-risk; recommend gating behind
`ENVIRONMENT != "production"`.

### M4. Uvicorn `reload=` decided by env default
**File:** `packages/api-gateway/src/main.py:252`

```python
reload=os.getenv("ENVIRONMENT") == "development"
```

Fine as-is, but note `ENVIRONMENT` is unset in the Dockerfile — verify
prod containers don't accidentally set it to `development`.

---

## Low / Informational

- **L1.** `BINANCE_SANDBOX=true` in `.env.example` is a safe default, good.
- **L2.** `packages/oms-engine/src/adapters/{binance,bybit}.rs` contain
  `api_key: "test_key"` — these are inside `#[cfg(test)]` module test
  fixtures, not production defaults. No action needed.
- **L3.** `.env.example` includes `POSTGRES_PASSWORD=change_me_in_production_to_secure_password`
  — explicit placeholder, docker-compose already `:?`-guards it. Good.

---

## Fixed in PR #4 (merged)

- C1 — hardcoded GitHub PAT removed from `scripts/`, `.github/mcp_github_setup.md`,
  `.windsurf/workflows/github-mcp-integration.md`, and `VALIDATION_STATUS.md`
- C2 — trading-engine CORS tightened; default bind moved to `127.0.0.1`
- C3 — model-serving CORS tightened
- C4 — Grafana admin password now required via `.env`
- H1 — API Gateway CORS tightened
- H3 — OMS observability server CORS driven by explicit config

---

## Follow-up round (second security PR)

This section tracks a second pass over the codebase after PR #4 merged.
It captures newly-discovered issues plus the follow-up items from PR #4
that are fixed in this round.

### C5. Hardcoded third-party data-provider API keys (NEW — critical)
**File:** `packages/quantbench/src/data/constants.py`

Three live credentials were committed in plaintext:

```python
EODHD_API_KEY = "64cab16..."           # truncated — see git history
POLYGON_IO_KEY = "NbOaYW..."           # truncated — see git history
AZURE_LANGUAGE_KEY = "0e9ec1..."       # truncated — see git history
```

`EODHD_API_KEY` is already `.format(...)`-interpolated into a URL in
`packages/quantbench/src/data/name_mapping.py:31`, so anyone with
repo read access can hit EODHD's API as this account. Polygon.io and
Azure Cognitive Services (at `https://aaai24.cognitiveservices.azure.com/`)
have the same exposure.

**Fix (this PR):** Constants are loaded from env vars; callers raise
a clear `RuntimeError` if a key is required but unset. Placeholders
added to `.env.example`.

**Owner action required (out-of-band):**
1. **Rotate all three keys.** They are in git history.
2. Revoke the EODHD token in the EODHD dashboard, regenerate, set
   `EODHD_API_KEY` in the session/org secret store.
3. Rotate the Polygon.io key at <https://polygon.io/dashboard/api-keys>.
4. Rotate the Azure Cognitive Services key in the Azure portal for
   the `aaai24` resource group, or (preferred) delete that resource
   entirely if it was a one-off.

### H5. `CORS_ALLOWED_ORIGINS` not forwarded to api-gateway container
**File:** `docker-compose.yml` (api-gateway service)

PR #4 taught `packages/api-gateway/src/main.py` to read
`CORS_ALLOWED_ORIGINS` at startup, but docker-compose only interpolates
`.env` into the compose file itself — it does **not** propagate those
variables into containers without either an `env_file:` directive or
an explicit `environment:` entry. Result: running the stack via
`docker compose up` silently ignored any `CORS_ALLOWED_ORIGINS`
override in `.env` and always used the hardcoded
`http://localhost:3000,https://traderx.com` fallback.

**Fix (this PR):** Added `CORS_ALLOWED_ORIGINS` (and `ENABLE_API_DOCS`)
to the api-gateway service's `environment:` block with documented
defaults.

### M3. FastAPI `/docs` + `/redoc` hidden by default
**File:** `packages/api-gateway/src/main.py:52-71`

`FastAPI(..., docs_url="/docs", redoc_url="/redoc")` was unconditional,
exposing the full OpenAPI schema + interactive Swagger UI on every
deployment — including production. That gives an attacker a
zero-effort map of every endpoint, request body, validator, and auth
requirement.

**Fix (this PR):** `docs_url`, `redoc_url`, and `openapi_url` are
conditioned on a new `ENABLE_API_DOCS` env var (default `false`). Local
developers set `ENABLE_API_DOCS=true` in `.env`; production leaves it
unset or explicitly `false`.

### M1. f-string table names in `tenant_manager.py`
**File:** `packages/database/src/tenant_manager.py:196-228`

Two admin helpers (`get_tenant_stats`, `migrate_existing_data`) built
SQL via `f"SELECT COUNT(*) FROM {table}"`. Tables came from a hardcoded
list so it wasn't exploitable today, but it's a future foot-gun if a
refactor ever sources the identifier from user input.

**Fix (this PR):** Introduced a module-level `frozenset` allow-list
(`_TENANT_SCOPED_TABLES`) and a `_assert_known_table(...)` helper that
raises `ValueError` on any identifier outside the allow-list. Both
callers now route their identifier through that helper before
interpolation, so the SQL surface cannot be widened without an
explicit code change in this module.

## Still outstanding (future PRs)

- **H2** — api-gateway endpoints still have no JWT auth; the rate
  limiter's tenant extraction uses the `"default-tenant"` shortcut.
  Proper auth is a larger change (token issuance, refresh, middleware,
  per-endpoint scopes) that deserves its own PR.
- **H4** — `pickle.loads` in vendored LAVIS
  (`packages/sugaformer/models/lavis/util/misc.py:146`,
  `packages/sugaformer/models/lavis/common/utils.py:333`). This is
  vendored third-party research code; the trust boundary needs to be
  documented (don't pass attacker-controlled pickle data) or the
  dependency replaced.
- **M2** — `subprocess(..., shell=True)` in
  `scripts/github_sync_check.py:20` and `scripts/self_healing_guard.py:33`
  plus three `clawteam` spawn/watch/hook files. All invocations are on
  static strings today (no interpolated user input) so not exploitable,
  but worth converting to list-form `subprocess.run(["git", ...])`
  as a defense-in-depth pass.
- **Dependency advisories** — `cargo audit` still reports
  `RUSTSEC-2024-0437` (protobuf 2.28.0) and
  `RUSTSEC-2026-0098/0099` (rustls-webpki 0.101.7) on transitive deps
  in `Cargo.lock`. Needs a dependency-bump PR.
- Token rotation — the PAT removed in C1 must be revoked by the repo owner
