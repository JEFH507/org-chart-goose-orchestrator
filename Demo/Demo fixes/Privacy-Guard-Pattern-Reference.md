# Privacy Guard Pattern Reference & UI Control Guide

**Document Version**: 1.0  
**Date**: 2025-11-16  
**Source**: `src/privacy-guard/src/detection.rs` (lines 90-280)

---

## Supported PII Patterns (8 Entity Types, 22 Patterns Total)

### 1. SSN (Social Security Number) - 3 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **Hyphenated** | `\b\d{3}-\d{2}-\d{4}\b` | HIGH | No | `123-45-6789` ✅ |
| **Spaced** | `\b\d{3}\s\d{2}\s\d{4}\b` | MEDIUM | No | `123 45 6789` ✅ |
| **No Separators** | `\b\d{9}\b` | LOW | Yes ("SSN", "social security", "SS#") | `123456789` ⚠️ |

**Test Data**:
```
My SSN is 123-45-6789  ✅ HIGH confidence
SSN: 987654321          ✅ LOW confidence (keyword "SSN" nearby)
The number 123456789    ❌ No keyword, won't detect
```

---

### 2. EMAIL - 1 Pattern

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **RFC-compliant** | `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b` | HIGH | No | `alice@company.com` ✅ |

**Test Data**:
```
Contact alice@company.com     ✅ Detects
Email: john.doe@example.org   ✅ Detects
```

---

### 3. PHONE - 5 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **Hyphenated US** | `\b\d{3}-\d{3}-\d{4}\b` | HIGH | No | `555-123-4567` ✅ |
| **Parentheses** | `\(\d{3}\)\s*\d{3}-\d{4}` | HIGH | No | `(555) 123-4567` ✅ |
| **Dotted** | `\b\d{3}\.\d{3}\.\d{4}\b` | HIGH | No | `555.123.4567` ✅ |
| **Country Code** | `\+1\s?\d{3}\s?\d{3}\s?\d{4}` | HIGH | No | `+1 555 123 4567` ✅ |
| **International** | `\+\d{1,3}\s?\d{4,14}` | MEDIUM | No | `+44 20 1234 5678` ✅ |

**Test Data**:
```
Call 555-123-4567           ✅ Detects
Phone: (555) 987-6543       ✅ Detects
International: +44 20 7946  ✅ Detects
```

---

### 4. CREDIT_CARD - 5 Patterns

| Pattern | Regex | Confidence | Luhn Check | Context Required | Example |
|---------|-------|------------|------------|------------------|---------|
| **Visa** | `\b4\d{15}\b` | HIGH | ✅ Yes | No | `4532015112830366` ✅ |
| **Mastercard** | `\b5[1-5]\d{14}\b` | HIGH | ✅ Yes | No | `5425233430109903` ✅ |
| **Amex** | `\b3[47]\d{13}\b` | HIGH | ✅ Yes | No | `378282246310005` ✅ |
| **Discover** | `\b6(?:011\|5\d{2})\d{12}\b` | HIGH | ✅ Yes | No | `6011000990139424` ✅ |
| **Generic Card** | `\b\d{13,19}\b` | MEDIUM | ✅ Yes | Yes ("card", "credit", "payment") | 13-19 digits ⚠️ |

**Important Notes**:
- ❌ **Hyphens NOT supported**: `4532-1234-5678-9012` will NOT match
- ✅ **Must be continuous digits**: `4532015112830366` will match
- ✅ **Luhn validation**: Invalid card numbers are ignored (prevents false positives)

**Test Data**:
```
Card: 4532015112830366              ✅ Valid Visa (passes Luhn)
Card: 4532-1234-5678-9012           ❌ Hyphens - won't match
Card: 1234567890123456              ❌ Invalid Luhn - won't match
Payment card 4532015112830366       ✅ Detects (context keyword "card")
```

**Valid Test Cards** (from unit tests):
```
4532015112830366  ✅ Valid Visa
5425233430109903  ✅ Valid Mastercard
378282246310005   ✅ Valid Amex
```

---

### 5. PERSON - 2 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **With Title** | `(?:Mr\.\|Mrs\.\|Ms\.\|Dr\.\|Prof\.)\s+[A-Z][a-z]+\s+[A-Z][a-z]+` | MEDIUM | No | `Dr. John Smith` ✅ |
| **Two Capitalized Words** | `\b[A-Z][a-z]+\s+[A-Z][a-z]+\b` | LOW | Yes ("name", "person", "employee", "contact", "from", "to", "by") | `Alice Johnson` ⚠️ |

**Test Data**:
```
Contact Dr. John Smith           ✅ MEDIUM confidence
Employee Alice Johnson           ✅ LOW confidence (keyword "employee")
Alice Johnson went there         ❌ No keyword, won't detect
```

---

### 6. IP_ADDRESS - 2 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **IPv4** | `\b(?:\d{1,3}\.){3}\d{1,3}\b` | HIGH | No | `192.168.1.100` ✅ |
| **IPv6** | `\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b` | HIGH | No | `2001:0db8:85a3:0000:0000:8a2e:0370:7334` ✅ |

**Test Data**:
```
Server IP: 192.168.1.100        ✅ Detects
External: 8.8.8.8               ✅ Detects
IPv6: 2001:db8::1               ⚠️ Abbreviated format may not match
```

---

### 7. DATE_OF_BIRTH - 2 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **With Label** | `(?:DOB\|Date of birth\|Born\|Birth date):\s*\d{1,2}/\d{1,2}/\d{2,4}` | HIGH | No | `DOB: 01/15/1985` ✅ |
| **Generic Date** | `\b\d{1,2}/\d{1,2}/\d{2,4}\b` | LOW | Yes ("birth", "DOB", "born", "age") | `12/25/2000` ⚠️ |

**Test Data**:
```
DOB: 01/15/1985                 ✅ HIGH confidence
Born on 12/25/2000              ✅ LOW confidence (keyword "born")
Meeting on 01/15/2025           ❌ No birth keyword, won't detect
```

---

### 8. ACCOUNT_NUMBER - 2 Patterns

| Pattern | Regex | Confidence | Context Required | Example |
|---------|-------|------------|------------------|---------|
| **With Label** | `(?:Account\|Acct\|Account #\|Acct #):\s*\d{8,16}` | HIGH | No | `Account #: 1234567890123456` ✅ |
| **Generic Number** | `\b\d{8,16}\b` | LOW | Yes ("account", "acct", "number", "ID") | `98765432` ⚠️ |

**Test Data**:
```
Account #: 1234567890123456     ✅ HIGH confidence
ID: 98765432                    ✅ LOW confidence (keyword "ID")
The number 12345678             ❌ No account keyword, won't detect
```

---

## Pattern Gaps & Limitations

### ❌ Not Supported

| Entity Type | Example Format | Why Not Supported |
|-------------|----------------|-------------------|
| **Employee ID** | `AB123456`, `EMP001` | No pattern exists in default rules |
| **Credit Card with Hyphens** | `4532-1234-5678-9012` | Regex requires continuous digits |
| **Credit Card with Spaces** | `4532 1234 5678 9012` | Regex requires continuous digits |
| **Abbreviated IPv6** | `2001:db8::1` | Regex only matches full format |
| **Dates with Hyphens** | `01-15-1985` | Only `/` separator supported |

### Workarounds

1. **Employee ID**: Would need to add custom pattern:
   ```rust
   Pattern {
       regex: Regex::new(r"\b[A-Z]{2,3}\d{5,8}\b").unwrap(),
       confidence: Confidence::MEDIUM,
       context_keywords: Some(vec!["employee".to_string(), "emp".to_string()]),
       description: "Employee ID (2-3 letters + 5-8 digits)".to_string(),
       luhn_check: false,
   }
   ```

2. **Credit Card with Separators**: Strip hyphens before detection (not currently implemented)

---

## Detection Methods & Performance

### 1. Rules-Only (Fast ~0-10ms)

**How it works**: Regex pattern matching only  
**Configured on**: Finance goose (`GUARD_MODEL_ENABLED=false`)  
**Pros**:
- Very fast (0-10ms per request)
- Deterministic results
- No external dependencies

**Cons**:
- Limited to exact pattern matches
- Misses context-dependent PII
- High false positive risk on LOW confidence patterns

**Best for**: High-throughput, structured data with known formats

---

### 2. AI-Only (Slow ~15s)

**How it works**: Ollama NER model (qwen3:0.6b)  
**Configured on**: Legal goose (`GUARD_MODEL_ENABLED=true`, Hybrid mode set to AI-only)  
**Pros**:
- Detects PII without exact patterns
- Context-aware (understands "my email" vs "email client")
- Handles variations (nicknames, informal names)

**Cons**:
- Very slow (~15s per request)
- Requires Ollama service
- Non-deterministic results

**Best for**: Unstructured text, legal documents, edge cases

---

### 3. Hybrid (Balanced ~100ms)

**How it works**: Regex first, then AI fallback/consensus  
**Configured on**: Manager goose (`GUARD_MODEL_ENABLED=true`)  
**Pros**:
- Best of both worlds
- Regex catches obvious patterns fast
- AI fills in gaps and validates
- Consensus → upgrades confidence to HIGH

**Cons**:
- Moderate latency (~100ms)
- Requires both services
- More complex logic

**Best for**: Production use with accuracy requirements

**Merge Logic** (from `detection.rs` lines 335-370):
```rust
// Consensus: Both regex AND model detect → HIGH confidence
// Model-only: AI found it, regex missed → HIGH confidence  
// Regex-only: Regex found it, AI missed → Keep original confidence
```

---

## UI Dashboard Controls

### Access

- **Finance Proxy**: http://localhost:8096/ui
- **Manager Proxy**: http://localhost:8097/ui
- **Legal Proxy**: http://localhost:8098/ui

### Settings Available

#### 1. Routing Mode (Level 1)
- **Service** (default): Route through Privacy Guard
- **Bypass**: Skip Privacy Guard entirely, go direct to LLM

#### 2. Detection Method (Level 2)
- **Rules** (default): Regex patterns only (~10ms)
- **AI**: Ollama NER model only (~15s)
- **Hybrid**: Both methods combined (~100ms)

#### 3. Privacy Mode (Level 2)
- **Auto** (default): Mask text content, bypass binary files with warning
- **Service-Bypass**: No masking, but still routed through service for audit
- **Strict**: Error on PII detection or unsupported content types

### Important Notes

#### ✅ Changes Apply Immediately (No Container Restart Needed)

**State is stored in-memory** (`src/privacy-guard-proxy/src/state.rs`):
```rust
pub struct ProxyState {
    pub routing_mode: Arc<RwLock<RoutingMode>>,      // In-memory
    pub current_mode: Arc<RwLock<PrivacyMode>>,      // In-memory
    pub detection_method: Arc<RwLock<DetectionMethod>>, // In-memory
    // ...
}
```

**How it works**:
1. User changes setting in UI (e.g., "Rules" → "Hybrid")
2. UI sends `PUT /api/settings` to Privacy Guard Proxy
3. Proxy updates in-memory state immediately
4. **Next goose request** uses the new setting
5. **No container restart required**

**Example Flow**:
```bash
# Initial state: Rules-only detection
curl http://localhost:8096/api/settings
# {"routing":"service","detection":"rules","privacy":"auto"}

# Change to Hybrid via UI (or curl)
curl -X PUT http://localhost:8096/api/settings \
  -H "Content-Type: application/json" \
  -d '{"routing":"service","detection":"hybrid","privacy":"auto"}'

# Next goose message uses Hybrid detection immediately
```

#### ⚠️ Settings Lost on Container Restart

**Not persisted to disk**:
- Settings revert to defaults on `docker restart ce_privacy_guard_proxy_finance`
- Defaults: `routing=service`, `detection=rules`, `privacy=auto`

**To make permanent**:
1. **Option A**: Set environment variables in `ce.dev.yml`:
   ```yaml
   privacy-guard-proxy-finance:
     environment:
       - DEFAULT_DETECTION_METHOD=hybrid  # Not currently implemented
   ```

2. **Option B**: Add persistence to `state.rs` (requires code change):
   ```rust
   // Save to file on change, load on startup
   pub async fn set_detection_method(&self, method: DetectionMethod) -> Result<(), String> {
       // ... existing logic ...
       self.save_to_file()?;  // New: persist to /config/settings.json
       Ok(())
   }
   ```

3. **Option C**: Use Controller Admin UI to push profile changes (currently placeholder button)

#### 🔒 Override Lock (Profile-Controlled)

**Default**: `allow_override: true` (users can change settings)

**If set to `false`** (by profile):
```bash
curl -X PUT http://localhost:8096/api/detection \
  -H "Content-Type: application/json" \
  -d '{"method":"hybrid"}'

# Response: 403 Forbidden
# {"error":"Detection method is locked by profile configuration"}
```

**Use case**: Enterprise deployment where finance dept MUST use rules-only for compliance

---

## Testing Privacy Guard Detection

### Test Script

Create `test-privacy-guard.sh`:
```bash
#!/bin/bash

# Test different PII patterns
echo "=== Testing Privacy Guard Detection ==="

# Test 1: SSN (HIGH confidence)
echo -e "\nTest 1: SSN with hyphens"
echo "Input: My SSN is 123-45-6789"
# Expected: {"entity_counts":{"SSN":1},"total_redactions":1}

# Test 2: Email (HIGH confidence)
echo -e "\nTest 2: Email address"
echo "Input: Contact alice@company.com"
# Expected: {"entity_counts":{"EMAIL":1},"total_redactions":1}

# Test 3: Credit Card (HIGH confidence, Luhn check)
echo -e "\nTest 3: Credit card (no hyphens)"
echo "Input: Card number 4532015112830366"
# Expected: {"entity_counts":{"CREDIT_CARD":1},"total_redactions":1}

# Test 4: Phone (HIGH confidence)
echo -e "\nTest 4: Phone number"
echo "Input: Call 555-123-4567"
# Expected: {"entity_counts":{"PHONE":1},"total_redactions":1}

# Test 5: Multiple entities
echo -e "\nTest 5: Mixed PII"
echo "Input: Contact Dr. John Smith at 555-123-4567 or john@example.com, SSN: 123-45-6789"
# Expected: {"entity_counts":{"PERSON":1,"PHONE":1,"EMAIL":1,"SSN":1},"total_redactions":4}

# Check audit logs
echo -e "\n=== Checking Audit Logs ==="
docker logs ce_privacy_guard_finance | grep audit | tail -5
```

### Verification Commands

```bash
# 1. Check current UI settings
curl http://localhost:8096/api/settings | jq

# 2. Check Privacy Guard status
curl http://localhost:8093/status | jq

# 3. Send test message via goose
docker exec -it ce_goose_finance goose session
# > My SSN is 123-45-6789 and card 4532015112830366

# 4. Check audit logs
docker logs ce_privacy_guard_finance | grep audit | tail -1 | jq

# 5. Verify LLM didn't see original values
# Ask: "What SSN did I send you?"
# Expected: LLM says it can't recall or won't repeat it
```

---

## Troubleshooting

### Issue: No PII Detected

**Check**:
1. **Pattern format**: Does your test data match the exact regex?
   ```bash
   # ❌ Won't match: 4532-1234-5678-9012 (hyphens)
   # ✅ Will match:  4532015112830366 (no separators)
   ```

2. **Context keywords**: LOW confidence patterns need keywords nearby
   ```bash
   # ❌ Won't match: "The number 123456789"
   # ✅ Will match:  "SSN: 123456789" (keyword "SSN")
   ```

3. **Luhn validation**: Invalid card numbers are rejected
   ```bash
   # Test if number passes Luhn check
   python3 -c "
   def luhn(n):
       digits = [int(d) for d in str(n)]
       checksum = 0
       for i, digit in enumerate(reversed(digits)):
           if i % 2 == 1:
               digit *= 2
               if digit > 9: digit -= 9
           checksum += digit
       return checksum % 10 == 0
   print(luhn(4532015112830366))  # True = valid
   "
   ```

### Issue: UI Changes Not Taking Effect

**Check**:
1. **Verify setting changed**:
   ```bash
   curl http://localhost:8096/api/settings | jq
   ```

2. **Check goose is using correct proxy**:
   ```bash
   docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep OPENROUTER_HOST
   # Should show: http://privacy-guard-proxy-finance:8090
   ```

3. **Check proxy logs for activity**:
   ```bash
   docker logs ce_privacy_guard_proxy_finance 2>&1 | grep detection_method_change
   ```

### Issue: Ollama Models Not Working

**Check**:
1. **Ollama service running**:
   ```bash
   curl http://localhost:11435/api/tags  # Finance Ollama
   ```

2. **Model pulled**:
   ```bash
   docker exec ce_ollama_finance ollama list
   # Should show: qwen3:0.6b
   ```

3. **Privacy Guard can reach Ollama**:
   ```bash
   docker exec ce_privacy_guard_finance curl -s http://ollama-finance:11434/api/tags
   ```

---

## Summary Table

| Entity Type | Total Patterns | HIGH Conf | MEDIUM Conf | LOW Conf | Luhn Check | Context Needed |
|-------------|----------------|-----------|-------------|----------|------------|----------------|
| SSN | 3 | 1 | 1 | 1 | No | 1 pattern |
| EMAIL | 1 | 1 | 0 | 0 | No | No |
| PHONE | 5 | 4 | 1 | 0 | No | No |
| CREDIT_CARD | 5 | 4 | 1 | 0 | Yes | 1 pattern |
| PERSON | 2 | 0 | 1 | 1 | No | 1 pattern |
| IP_ADDRESS | 2 | 2 | 0 | 0 | No | No |
| DATE_OF_BIRTH | 2 | 1 | 0 | 1 | No | 1 pattern |
| ACCOUNT_NUMBER | 2 | 1 | 0 | 1 | No | 1 pattern |
| **TOTAL** | **22** | **14** | **4** | **4** | **5** | **5** |

---

## References

- **Source Code**: `src/privacy-guard/src/detection.rs`
- **UI Code**: `src/privacy-guard-proxy/src/control_panel.rs`
- **State Management**: `src/privacy-guard-proxy/src/state.rs`
- **Docker Compose**: `deploy/compose/ce.dev.yml`
- **Demo Guide**: `Demo/DEMO_GUIDE.md`
- **Validation State**: `Demo/Demo-Validation-State.json`
