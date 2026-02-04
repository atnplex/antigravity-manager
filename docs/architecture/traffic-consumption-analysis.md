# API Proxy Traffic Consumption Mechanism

> Translated and consolidated from original Chinese documentation.

## Overview

Antigravity Manager is a Tauri-based desktop application providing reverse proxy for Google AI (Gemini/Claude) APIs. It manages multiple Google accounts for intelligent traffic distribution and quota management.

---

## 1. Core Architecture

### 1.1 Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust (Tauri) + Axum (Web Framework)
- **Data Storage**:
  - SQLite (traffic stats, proxy logs, IP monitoring)
  - JSON files (account info, config)

### 1.2 Core Modules

```
src-tauri/src/
├── proxy/
│   ├── token_manager.rs      # Token pool management & scheduling
│   ├── server.rs             # Axum HTTP server
│   ├── rate_limit.rs         # Rate limit tracker
│   └── sticky_config.rs      # Session stickiness config
├── modules/
│   ├── account.rs            # Account CRUD operations
│   ├── quota.rs              # Quota query & protection
│   └── token_stats.rs        # Token usage statistics
└── models/
    ├── account.rs            # Account data model
    ├── quota.rs              # Quota data model
    └── token.rs              # Token data model
```

---

## 2. Traffic Consumption Mechanism

### 2.1 Account Data Structure

```rust
pub struct Account {
    pub id: String,                          // Unique account ID
    pub email: String,                       // Google email
    pub token: TokenData,                    // OAuth token info
    pub quota: Option<QuotaData>,            // Quota info
    pub disabled: bool,                      // Globally disabled
    pub proxy_disabled: bool,                // Proxy disabled
    pub protected_models: HashSet<String>,   // Quota-protected models
    pub created_at: i64,
    pub last_used: i64,
}
```

**Token Data**:

```rust
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub expiry_timestamp: i64,
    pub project_id: Option<String>,
}
```

**Quota Data**:

```rust
pub struct QuotaData {
    pub models: Vec<ModelQuota>,
    pub subscription_tier: Option<String>,  // FREE/PRO/ULTRA
    pub is_forbidden: bool,
}

pub struct ModelQuota {
    pub name: String,
    pub percentage: i32,      // Remaining quota 0-100
    pub reset_time: String,   // Quota reset time
}
```

### 2.2 Token Pool Management (TokenManager)

**Core Responsibilities**:

1. Load all available accounts from disk
2. Intelligently select accounts for requests
3. Automatically refresh expired tokens
4. Track rate limits and quota protection

**Key Fields**:

```rust
pub struct TokenManager {
    tokens: Arc<DashMap<String, ProxyToken>>,       // Account ID -> Token mapping
    current_index: Arc<AtomicUsize>,                // Round-robin index
    last_used_account: Arc<Mutex<Option<(String, Instant)>>>, // 60s lock mechanism
    rate_limit_tracker: Arc<RateLimitTracker>,      // Rate limit tracker
    session_accounts: Arc<DashMap<String, String>>, // Session -> Account binding
    preferred_account_id: Arc<RwLock<Option<String>>>, // Fixed account mode
    health_scores: Arc<DashMap<String, f32>>,       // Account health scores
}
```

### 2.3 Account Scheduling Strategy

#### Priority Sorting

Accounts are sorted by these priorities on each request:

```rust
// 1. Subscription tier: ULTRA > PRO > FREE
// 2. Within same tier: sort by remaining quota descending
// 3. Same quota: sort by health score descending

tokens_snapshot.sort_by(|a, b| {
    let tier_priority = |tier: &Option<String>| match tier.as_deref() {
        Some("ULTRA") => 0,
        Some("PRO") => 1,
        Some("FREE") => 2,
        _ => 3,
    };

    let tier_cmp = tier_priority(&a.subscription_tier)
        .cmp(&tier_priority(&b.subscription_tier));

    if tier_cmp != std::cmp::Ordering::Equal {
        return tier_cmp;
    }

    // Prefer higher quota
    let quota_cmp = b.remaining_quota.cmp(&a.remaining_quota);
    if quota_cmp != std::cmp::Ordering::Equal {
        return quota_cmp;
    }

    // Prefer higher health score
    b.health_score.partial_cmp(&a.health_score)
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

**Rationale**:

- **ULTRA/PRO first**: Faster quota reset, maximize availability
- **High quota first**: Preserve low-quota accounts
- **Health score**: Dynamic adjustment based on success rate

#### Scheduling Modes

1. **CacheFirst** (Cache Priority)
   - 60s global lock enabled
   - Session stickiness enabled
   - Best for context-continuity scenarios

2. **Balance** (Balanced)
   - Session stickiness enabled
   - No 60s lock
   - Balance between performance and continuity

3. **PerformanceFirst** (Performance Priority)
   - Pure round-robin
   - No locking mechanisms
   - Maximum concurrent performance

### 2.4 Quota Protection Mechanism

#### Model-Level Protection

When a model's quota falls below threshold, it's added to `protected_models`:

```json
{
  "quota_protection": {
    "enabled": true,
    "threshold_percentage": 10,
    "monitored_models": [
      "gemini-3-flash",
      "claude-sonnet-4-5",
      "gemini-3-pro-high"
    ]
  }
}
```

**Protection Flow**:

1. Check each model's quota when loading account
2. If `percentage <= threshold_percentage`, add to `protected_models`
3. In `get_token()`, skip accounts with protected target model
4. Auto-remove from `protected_models` when quota recovers

**Advantage**: Granular protection - won't disable entire account for one low-quota model.

### 2.5 Rate Limit Tracking

```rust
pub struct RateLimitTracker {
    // account_id -> (reset_timestamp, model_name)
    records: Arc<DashMap<String, (i64, Option<String>)>>,
    // account_id -> 5xx error count
    error_counts: Arc<DashMap<String, u32>>,
}

// Usage
tracker.record_rate_limit(&account_id, Some(&model_name), 60);
if tracker.is_rate_limited(&account_id, Some(&model_name)) {
    // Skip this account
}
tracker.clear(&account_id);
```

### 2.6 Session Stickiness

**Purpose**: Keep same session using same account for context continuity.

```rust
// First request: bind session to account
if let Some(sid) = session_id {
    self.session_accounts.insert(sid.to_string(), account_id.clone());
}

// Subsequent requests: reuse bound account
if let Some(bound_id) = self.session_accounts.get(sid) {
    if let Some(bound_token) = tokens_snapshot.iter().find(|t| t.account_id == bound_id) {
        if !is_rate_limited && !is_quota_protected {
            target_token = Some(bound_token.clone());
        } else {
            // Account unavailable, unbind and switch
            self.session_accounts.remove(sid);
        }
    }
}
```

### 2.7 Token Auto-Refresh

**Trigger**: Token expires in less than 5 minutes.

```rust
let now = chrono::Utc::now().timestamp();
if now >= token.timestamp - 300 {  // 5 minutes before expiry
    match refresh_access_token(&token.refresh_token).await {
        Ok(token_response) => {
            // Update in-memory token
            token.access_token = token_response.access_token.clone();

            // Sync to DashMap
            if let Some(mut entry) = self.tokens.get_mut(&token.account_id) {
                entry.access_token = token.access_token.clone();
            }

            // Persist to disk
            self.save_refreshed_token(&token.account_id, &token_response).await?;
        }
        Err(e) if e.contains("invalid_grant") => {
            // Refresh token invalid, disable account
            self.disable_account(&token.account_id, &format!("invalid_grant: {}", e)).await?;
            self.tokens.remove(&token.account_id);
        }
    }
}
```

---

## 3. Request Allocation Flow

```
Client Request
    ↓
[1] Parse request params (model, session_id)
    ↓
[2] Fixed account mode check
    ├─ Has preferred_account_id → Use it
    └─ No → Continue
    ↓
[3] Session stickiness check (if session_id present)
    ├─ Already bound → Check availability
    │   ├─ Available → Reuse
    │   └─ Unavailable → Unbind, continue
    └─ Not bound → Continue
    ↓
[4] 60s global lock check (CacheFirst mode)
    ├─ <60s since last use → Reuse last account
    └─ Otherwise → Continue
    ↓
[5] Round-robin account selection
    ├─ Sort by priority
    ├─ Skip already-tried accounts
    ├─ Skip rate-limited accounts
    ├─ Skip quota-protected accounts
    └─ Select first available
    ↓
[6] Token refresh check
    ├─ <5 min to expiry → Refresh
    └─ Otherwise → Continue
    ↓
[7] Return (access_token, project_id, email)
    ↓
[8] Proxy request to Google API
    ↓
[9] Record usage stats
    ├─ token_stats.db
    ├─ proxy_logs.db
    └─ Update last_used timestamp
```

---

## 4. Traffic Statistics

### Database Structure

**token_usage table** (Raw records):

```sql
CREATE TABLE token_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    account_email TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0
);
```

**token_stats_hourly table** (Hourly aggregation):

```sql
CREATE TABLE token_stats_hourly (
    hour_bucket TEXT NOT NULL,
    account_email TEXT NOT NULL,
    total_input_tokens INTEGER NOT NULL DEFAULT 0,
    total_output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    request_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (hour_bucket, account_email)
);
```

---

## 5. Optimizations

### Performance

1. **DashMap**: Lock-free concurrent HashMap
2. **Pre-sorting**: Sort accounts before request
3. **Batch operations**: Concurrent quota refresh
4. **Connection pooling**: HTTP client connection reuse

### Reliability

1. **Token auto-refresh**: 5 minutes before expiry
2. **Rate limit auto-skip**: Proactive detection
3. **Quota protection**: Auto-protect low-quota accounts
4. **Circuit breaker**: Disable accounts after 5xx threshold

### User Experience

1. **Session stickiness**: Conversation continuity
2. **Smart scheduling**: Tier and quota-based allocation
3. **Real-time stats**: Detailed usage trends
4. **Quota visualization**: Real-time quota status display

---

## 6. Typical Use Cases

### High Concurrency API Calls

- Mode: `PerformanceFirst`
- Accounts: 10+ (ULTRA/PRO recommended)
- Quota protection: Enabled, 10% threshold

### Long Conversations

- Mode: `CacheFirst` or `Balance`
- Session stickiness: Enabled
- 60s lock: Enabled (CacheFirst)

### Quota Protection

- Protection: Enabled, 10% threshold
- Monitored models: `["gemini-3-flash", "claude-sonnet-4-5"]`
- Auto-refresh: Hourly

---

*Document Version: 2.0 (English)*
*Translated: 2026-02-04*
*Original: API反代流量消耗机制分析.md*
