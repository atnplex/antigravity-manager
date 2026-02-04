# Network Architecture & Risk Analysis

> Translated and consolidated from original Chinese documentation.

## 1. Network Traffic Architecture

### 1.1 Default Architecture: Centralized Single Node

By default, **all account traffic routes through a single IP node**:

```
Client Application
    ↓
Your Reverse Proxy Server (127.0.0.1:8045 or 0.0.0.0:8045)
    ↓
[Optional] Upstream Proxy
    ↓
Google API (googleapis.com)
```

#### Key Configuration

**1. Local Binding Address** (`proxy.allow_lan_access`)

```rust
pub fn get_bind_address(&self) -> &str {
    if self.allow_lan_access {
        "0.0.0.0"  // Allow LAN access
    } else {
        "127.0.0.1"  // Localhost only (default)
    }
}
```

**2. Upstream Proxy** (`proxy.upstream_proxy`)

```rust
pub struct UpstreamProxyConfig {
    pub enabled: bool,
    pub url: String,  // Supports http://, https://, socks5://
}
```

### 1.2 Traffic Flow Scenarios

#### Scenario A: Local Usage (Default)

```
Your Computer (127.0.0.1)
  ├─ Claude Code CLI
  ├─ Cherry Studio
  └─ Python Scripts
      ↓
Antigravity Proxy (127.0.0.1:8045)
  └─ Account Pool (10 Google accounts)
      ↓
Your Network Exit IP (Single IP)
      ↓
Google API
```

**Characteristics**:

- ✅ All accounts share single exit IP
- ✅ Simple, no extra configuration
- ⚠️ IP association risk exists

#### Scenario B: LAN Sharing

```
LAN Devices
  ├─ Device A (192.168.1.100)
  ├─ Device B (192.168.1.101)
  └─ Device C (192.168.1.102)
      ↓
Your Server (192.168.1.50:8045, allow_lan_access=true)
  └─ Account Pool (10 Google accounts)
      ↓
Your Network Exit IP (Single IP)
      ↓
Google API
```

#### Scenario C: Distributed via Upstream Proxy

```
Antigravity Proxy
  └─ Account Pool (10 Google accounts)
      ↓
Upstream Proxy Pool
  ├─ Proxy 1 (IP: 1.2.3.4)
  ├─ Proxy 2 (IP: 5.6.7.8)
  └─ Proxy 3 (IP: 9.10.11.12)
      ↓
Google API
```

### 1.3 Current Limitations

The `upstream_proxy` is **global** - all accounts share one proxy:

```rust
fn create_base_client(timeout_secs: u64) -> Client {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs));

    if let Ok(config) = load_app_config() {
        let proxy_config = config.proxy.upstream_proxy;  // Global config
        if proxy_config.enabled && !proxy_config.url.is_empty() {
            match Proxy::all(&proxy_config.url) {  // All requests use same proxy
                Ok(proxy) => { builder = builder.proxy(proxy); }
                Err(e) => { tracing::error!("invalid_proxy_url: {}", e); }
            }
        }
    }

    builder.build().unwrap_or_else(|_| Client::new())
}
```

**Conclusion**:

- ❌ Cannot configure per-account proxies
- ❌ Cannot achieve true distributed IPs
- ✅ Can use external proxy pool + rotation (requires additional development)

---

## 2. Risk Analysis

### 2.1 Operator Perspective (Proxy Service Provider)

#### Risk Level: 🟡 Medium

**Primary Risks**:

1. **IP Association Risk**
   - **Issue**: Multiple Google accounts making requests from same IP
   - **Effect**: Google may detect anomalous traffic patterns
   - **Consequences**:
     - 429 rate limiting (short-term)
     - Account flagged as "suspicious activity" (medium-term)
     - Potential account ban (long-term)

2. **Traffic Fingerprint Risk**
   - **Issue**: All requests have identical User-Agent, TLS fingerprint
   - **Code evidence**:

     ```rust
     pub static USER_AGENT: LazyLock<String> = LazyLock::new(|| {
         format!("antigravity/{}", get_version())
     });
     ```

   - **Consequence**: Easy to identify as automation tool

3. **Device Fingerprint Risk**
   - **Good news**: Device fingerprint isolation is implemented
   - **Code evidence**:

     ```rust
     pub struct Account {
         pub device_profile: Option<DeviceProfile>,
         pub device_history: Vec<DeviceProfileVersion>,
     }

     pub struct DeviceProfile {
         pub machine_id: String,
         pub mac_machine_id: String,
         pub dev_device_id: String,
         pub sqm_id: String,
     }
     ```

   - **Effect**: Each account can simulate different device, reducing association risk

#### Implemented Mitigations

1. ✅ Device fingerprint isolation per account
2. ✅ Smart scheduling to avoid frequent account switching
3. ✅ Rate limit protection with automatic backoff
4. ✅ Quota protection with per-model thresholds

#### Recommended Enhancements

1. 🔧 IP pooling (per-account proxy)
2. 🔧 User-Agent randomization
3. 🔧 Request frequency control

### 2.2 User Perspective (API Consumer)

#### Risk Level: 🟢 Low

**User View**:

```
User (Client)
    ↓
Your Proxy (http://your-server:8045)
    ↓
[Black Box] Account Pool + Scheduling
    ↓
Google API
```

**Risk Analysis**:

1. **IP Exposure**: ❌ None - user's real IP never exposed to Google
2. **Account Association**: ❌ None - user doesn't know which account is used
3. **Data Privacy**: ⚠️ Medium - all requests pass through your server
4. **Service Stability**: 🟡 Medium - if your server is blocked, all users affected

---

## 3. Google Detection Mechanisms

### 3.1 Detection Dimensions

1. **IP Dimension**
   - High request volume from single IP
   - IP geolocation mismatch with account registration
   - Datacenter IPs (higher risk than residential)

2. **Account Dimension**
   - Frequent login location changes
   - Abnormal behavior patterns (24/7 non-stop usage)
   - Multiple accounts with highly similar behavior

3. **Device Dimension**
   - Device fingerprint (machine_id, device_id)
   - User-Agent mismatch with device
   - TLS fingerprint anomalies

4. **Traffic Dimension**
   - Abnormal request frequency
   - Mechanical patterns (fixed intervals)
   - API volume far exceeding normal users

### 3.2 Antigravity Countermeasures

**Implemented**:

- Device fingerprint isolation
- Smart scheduling algorithm
- Rate limit protection
- Quota protection

**Missing**:

- IP pooling
- User-Agent diversity
- Request time randomization
- Human behavior simulation

---

## 4. Risk Level Assessment

### Usage Scenario Risk Levels

| Scenario | Accounts | Daily Requests | IP Strategy | Risk Level | Recommendation |
|----------|----------|----------------|-------------|------------|----------------|
| **Personal** | 1-3 | <1000 | Single IP | 🟢 Low | Normal usage |
| **Small Team** | 3-10 | 1000-5000 | Single IP | 🟡 Medium | Enable device isolation |
| **Commercial** | 10-50 | 5000-20000 | Single IP | 🔴 High | **Must** use IP pool + device isolation |
| **Large Scale** | 50+ | 20000+ | Single IP | 🔴 Critical | **Prohibited** - requires distributed architecture |

### Typical Detection Thresholds (Empirical)

| Metric | Safe | Warning | Danger |
|--------|------|---------|--------|
| **Daily requests per IP** | <5000 | 5000-10000 | >10000 |
| **QPS per account** | <1 | 1-3 | >3 |
| **Account switches/hour** | <10 | 10-30 | >30 |
| **Accounts per IP** | <5 | 5-10 | >10 |

---

## 5. Best Practices

### 5.1 Low Risk (Personal/Small Team)

```json
{
  "proxy": {
    "enabled": true,
    "allow_lan_access": false,
    "port": 8045,
    "scheduling": {
      "mode": "CacheFirst",
      "enable_sticky": true
    }
  },
  "quota_protection": {
    "enabled": true,
    "threshold_percentage": 20
  }
}
```

### 5.2 Medium Risk (Commercial)

```json
{
  "proxy": {
    "enabled": true,
    "allow_lan_access": true,
    "upstream_proxy": {
      "enabled": true,
      "url": "http://proxy-pool.example.com:8080"
    },
    "scheduling": {
      "mode": "Balance"
    }
  }
}
```

**Requirements**:

- **Must** use residential proxy pool
- **Must** bind device fingerprints per account
- Implement request rate limiting (QPS <1 per account)
- Monitor 429 error rates

### 5.3 High Risk (Large Scale)

**Architecture Upgrade Required**:

```
Users
    ↓
Load Balancer
    ↓
Multiple Antigravity Instances
  ├─ Instance 1 (Accounts 1-10, Proxy Pool A)
  ├─ Instance 2 (Accounts 11-20, Proxy Pool B)
  └─ Instance 3 (Accounts 21-30, Proxy Pool C)
      ↓
Google API
```

---

## 6. Summary

### Strengths

- ✅ Device fingerprint isolation
- ✅ Smart scheduling algorithm
- ✅ Rate limit protection
- ✅ Quota protection

### Weaknesses

- ❌ All accounts share single IP
- ❌ Cannot configure per-account proxies
- ❌ Fixed User-Agent
- ❌ No request frequency limiting

### Your Risk Level Depends On

1. **Account count**: More = higher risk
2. **Request volume**: More = higher risk
3. **IP strategy**: Single IP = high risk, IP pool = low risk
4. **Usage scenario**: Personal = low risk, commercial = high risk

---

*Document Version: 2.0 (English)*
*Translated: 2026-02-04*
*Original: 网络架构与风控分析.md*
