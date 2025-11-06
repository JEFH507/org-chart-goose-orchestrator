# Privacy Guard User Override UI Mockup

**Version:** 1.0.0  
**Date:** 2025-11-06  
**Status:** Design Specification  
**Target:** Goose Desktop Client (v1.13.0+)

---

## Overview

This document specifies the UI design for user-controlled Privacy Guard overrides in the Goose Desktop application. Users can temporarily adjust privacy settings per session without requiring admin intervention.

### Design Principles

1. **User Empowerment:** Users control their own privacy preferences
2. **Transparency:** Clear indication when privacy is relaxed/strengthened
3. **Temporary Overrides:** Changes apply to current session only
4. **Visual Clarity:** Obvious privacy status indicators
5. **Minimal Friction:** Quick toggles for common use cases

---

## UI Location

**Access Path:** Settings → Privacy & Security → Privacy Guard Settings

**Menu Structure:**
```
Goose Desktop
├── Chat (main view)
├── Sessions
├── Extensions
└── Settings
    ├── General
    ├── Providers
    ├── Extensions
    ├── Privacy & Security ← NEW SECTION
    │   ├── Privacy Guard Settings ← THIS PAGE
    │   ├── Memory Settings
    │   └── Data Retention
    ├── Keyboard Shortcuts
    └── About
```

---

## Wireframe: Privacy Guard Settings Page

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Goose Desktop                                            [−] [□] [×]     │
├──────────────────────────────────────────────────────────────────────────┤
│  ← Settings                                                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Privacy & Security > Privacy Guard Settings                            │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ 🔒 Privacy Guard Status                                            │ │
│  │                                                                     │ │
│  │  Status: ● Active (Hybrid Mode)                                    │ │
│  │  Profile: Finance (Strict)                                         │ │
│  │  Override: Session Only                                            │ │
│  │                                                                     │ │
│  │  ⚠️  Your admin has set privacy to "Strict" - overrides are       │ │
│  │      temporary and apply to this session only.                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Privacy Mode                                                        │ │
│  │                                                                     │ │
│  │  ○ Off            ⚠️  No PII protection (not recommended)          │ │
│  │  ○ Rules Only     📋 Fast regex-based detection                    │ │
│  │  ● Hybrid Mode    🤖 Rules + AI (NER) - recommended                │ │
│  │  ○ NER Only       🧠 AI-only detection (slower)                    │ │
│  │                                                                     │ │
│  │  ℹ️  Profile default: Hybrid Mode                                  │ │
│  │  ℹ️  Current session: Hybrid Mode (unchanged)                      │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Privacy Strictness                                                  │ │
│  │                                                                     │ │
│  │  [━━━●━━━━━━] Strict                                               │ │
│  │   Permissive ←──────────→ Strict                                   │ │
│  │                                                                     │ │
│  │  ● Permissive  Allow most tasks, redact obvious PII only           │ │
│  │  ● Moderate    Balance usability and privacy (recommended)         │ │
│  │  ● Strict      Maximum protection, may block some tasks            │ │
│  │                                                                     │ │
│  │  ℹ️  Profile default: Strict                                       │ │
│  │  ℹ️  Current session: Strict (unchanged)                           │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ PII Categories (Advanced)                                           │ │
│  │                                                                     │ │
│  │  Select which PII types to redact:                                 │ │
│  │                                                                     │ │
│  │  ☑ Social Security Numbers (SSN)                                   │ │
│  │  ☑ Email Addresses                                                 │ │
│  │  ☑ Phone Numbers                                                   │ │
│  │  ☑ Credit Card Numbers                                             │ │
│  │  ☑ Person Names (NER)                                              │ │
│  │  ☑ Organization Names (NER)                                        │ │
│  │  ☑ Locations/Addresses (NER)                                       │ │
│  │  ☑ IP Addresses                                                    │ │
│  │                                                                     │ │
│  │  🔗 Show Advanced Patterns...                                      │ │
│  │                                                                     │ │
│  │  ⚠️  All categories enabled by profile (Finance/Strict)            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Session Overrides                                                   │ │
│  │                                                                     │ │
│  │  ☑ Allow temporary privacy reduction for this session              │ │
│  │                                                                     │ │
│  │  Duration:  ○ Current chat only                                    │ │
│  │             ● Until I close Goose (session ends)                   │ │
│  │             ○ For 1 hour                                            │ │
│  │             ○ For 4 hours                                           │ │
│  │                                                                     │ │
│  │  Justification (optional):                                          │ │
│  │  ┌────────────────────────────────────────────────────────────┐   │ │
│  │  │ Debugging production issue - need to share error logs      │   │ │
│  │  └────────────────────────────────────────────────────────────┘   │ │
│  │                                                                     │ │
│  │  ℹ️  Overrides are logged for audit purposes                       │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Audit Log                                                           │ │
│  │                                                                     │ │
│  │  View privacy events for this session:                             │ │
│  │                                                                     │ │
│  │  2025-11-06 04:15:22 - 3 SSNs redacted (SSN → [SSN_XXX])           │ │
│  │  2025-11-06 04:12:45 - 2 emails redacted (email → [EMAIL_XXX])     │ │
│  │  2025-11-06 04:10:11 - Privacy mode changed: Hybrid → Rules Only   │ │
│  │  2025-11-06 04:05:33 - Session started (Finance profile)           │ │
│  │                                                                     │ │
│  │  [View Full Audit Log →]                                           │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │              [Reset to Profile Defaults]    [Apply Changes]        │ │
│  │                                                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Component Specifications

### 1. Privacy Guard Status Panel

**Purpose:** Show current privacy configuration at a glance

**Elements:**
- **Status Indicator:** Green dot (● Active) or Red dot (○ Inactive)
- **Mode Display:** Current privacy mode (Off/Rules/Hybrid/NER)
- **Profile Name:** Which role profile is active (Finance, Legal, etc.)
- **Override Indicator:** "Session Only" or "Profile Default"
- **Warning Banner:** If admin has restrictions (e.g., "Strict, no override allowed")

**States:**
```
✅ Active (Hybrid Mode) - Normal operation
⚠️  Active (Override: Permissive) - User reduced privacy
❌ Inactive - Privacy Guard disabled
🔒 Locked (Admin) - User cannot change settings
```

---

### 2. Privacy Mode Selector

**Purpose:** Choose detection method

**Options:**

| Mode | Icon | Description | Use Case |
|------|------|-------------|----------|
| **Off** | ⚠️ | No PII protection | Local-only work (Legal profile) |
| **Rules Only** | 📋 | Fast regex patterns | Quick tasks, minimal latency |
| **Hybrid** | 🤖 | Rules + NER | Recommended (balance speed/accuracy) |
| **NER Only** | 🧠 | AI-based detection | Maximum accuracy (slower) |

**Visual Design:**
- Radio buttons (single selection)
- Profile default shown below in info box
- Current session value highlighted if different from default

**Behavior:**
- If profile sets `allow_override: false`, options are **disabled** (grayed out)
- If override allowed, user can select any mode
- Changes apply immediately to current session

---

### 3. Privacy Strictness Slider

**Purpose:** Adjust redaction aggressiveness

**Visual Design:**
- Horizontal slider with 3 labeled stops
- Color-coded: Green (Permissive) → Yellow (Moderate) → Red (Strict)
- Current position shows active level

**Levels:**

| Level | Threshold | Behavior | Example |
|-------|-----------|----------|---------|
| **Permissive** | 90% confidence | Only obvious PII redacted | "John Smith" (common name) → passes through |
| **Moderate** | 70% confidence | Balance usability/privacy | "John Smith" → `[PERSON_A]` |
| **Strict** | 50% confidence | Maximum protection | "John" → `[PERSON_A]`, "Smith" → `[PERSON_B]` |

**Interaction:**
- Click slider to jump to level
- Drag knob to adjust
- Info boxes update dynamically to show profile default vs current

---

### 4. PII Categories (Advanced)

**Purpose:** Fine-grained control over what gets redacted

**Categories (Checkboxes):**

| Category | Pattern Type | Example Input | Redacted Output |
|----------|--------------|---------------|-----------------|
| **SSN** | Regex | 123-45-6789 | `[SSN_ABC]` |
| **Email** | Regex | user@example.com | `[EMAIL_XYZ]` |
| **Phone** | Regex | (555) 123-4567 | `[PHONE_ABC]` |
| **Credit Card** | Regex | 4111-1111-1111-1111 | `[CC_XXXX]` |
| **Person Names** | NER | John Smith | `[PERSON_A]` |
| **Organizations** | NER | Acme Corp | `[ORG_A]` |
| **Locations** | NER | 123 Main St | `[LOCATION_A]` |
| **IP Addresses** | Regex | 192.168.1.1 | `[IP_XXX]` |

**Advanced Patterns Expansion:**
- Click "Show Advanced Patterns..." to reveal custom regex editor
- Modal dialog with pattern list, test input field
- Add/Edit/Delete custom patterns

**Profile Override Logic:**
```
If profile.allow_override == false:
    All checkboxes disabled (grayed out)
    Warning: "Categories locked by admin"
Else:
    User can enable/disable categories
    Changes apply to current session only
```

---

### 5. Session Overrides Panel

**Purpose:** Manage temporary privacy changes

**Elements:**

#### A. Override Checkbox
- **Label:** "Allow temporary privacy reduction for this session"
- **Default:** Unchecked (profile settings enforced)
- **Effect:** Enables duration selector and justification field

#### B. Duration Radio Buttons
```
○ Current chat only          (override ends when chat thread closed)
● Until I close Goose         (override ends when app closed)
○ For 1 hour                  (auto-revert after 1 hour)
○ For 4 hours                 (auto-revert after 4 hours)
```

#### C. Justification Text Field
- **Placeholder:** "Why are you reducing privacy? (logged for audit)"
- **Max Length:** 500 characters
- **Optional:** Can be left blank (but discouraged)
- **Example:** "Debugging production issue - need to share error logs with support team"

#### D. Audit Warning
- **Text:** "ℹ️ Overrides are logged for audit purposes"
- **Tooltip:** "Your admin can see when you reduce privacy, what changed, and your justification"

**Workflow:**
1. User checks "Allow temporary privacy reduction"
2. Duration selector activates
3. User selects duration (default: "Until I close Goose")
4. User enters justification (optional but recommended)
5. User clicks "Apply Changes"
6. Privacy settings change, audit log entry created
7. After duration expires, settings auto-revert to profile defaults

---

### 6. Audit Log Panel

**Purpose:** Show recent privacy events for transparency

**Display Format:**
```
YYYY-MM-DD HH:MM:SS - [Event Description]
```

**Event Types:**
- **Redaction Events:** "N [category] redacted ([example] → [token])"
- **Mode Changes:** "Privacy mode changed: [old] → [new]"
- **Override Events:** "Override enabled: [reason]"
- **Session Events:** "Session started ([profile] profile)"

**Example Entries:**
```
2025-11-06 04:15:22 - 3 SSNs redacted (SSN → [SSN_XXX])
2025-11-06 04:12:45 - 2 emails redacted (email → [EMAIL_XXX])
2025-11-06 04:10:11 - Privacy mode changed: Hybrid → Rules Only
2025-11-06 04:05:33 - Session started (Finance profile)
```

**Interaction:**
- Shows last 5 events inline
- "View Full Audit Log →" button opens modal with full history
- Modal has filter/search, export to CSV

---

## User Workflows

### Workflow 1: Quick Privacy Reduction (Temporary)

**Scenario:** Finance user needs to share raw error logs with support team for 1 hour

**Steps:**
1. User opens Settings → Privacy & Security → Privacy Guard Settings
2. Status shows: "Finance (Strict), Hybrid Mode"
3. User checks "Allow temporary privacy reduction for this session"
4. User selects duration: "For 1 hour"
5. User enters justification: "Sharing error logs with support team for ticket #12345"
6. User clicks "Apply Changes"
7. **Result:** 
   - Privacy Guard mode stays "Hybrid" but strictness → "Permissive"
   - SSN category disabled (error logs may contain test SSNs)
   - Audit log created: "Override enabled (1h): Sharing error logs..."
   - Toast notification: "Privacy reduced for 1 hour. Audit logged."
8. **After 1 hour:** Auto-revert to profile defaults, toast: "Privacy restored to profile settings"

---

### Workflow 2: Legal User (Local-Only, No Override)

**Scenario:** Legal user wants to change privacy settings but profile is locked

**Steps:**
1. User opens Privacy Guard Settings
2. Status shows: "🔒 Locked by Admin - Legal (Strict, Local-Only)"
3. All controls grayed out
4. Warning: "Your admin has locked privacy settings for the Legal role. Changes are not allowed."
5. User clicks "Contact Admin" link (opens email to admin)
6. **Result:** No changes allowed, user must contact admin

---

### Workflow 3: View Audit Log

**Scenario:** User wants to see what PII was redacted in current session

**Steps:**
1. User opens Privacy Guard Settings
2. Scroll to "Audit Log" panel
3. View last 5 events inline
4. Click "View Full Audit Log →"
5. Modal opens showing all events with filter/search/export options
6. **Result:** Full transparency into privacy actions

---

## API Integration

### Load Profile Settings
```http
GET /profiles/{role}

Response:
{
  "role": "finance",
  "privacy": {
    "mode": "Hybrid",
    "strictness": "Strict",
    "categories": ["ssn", "email", "phone", "cc", "person", "org", "location", "ip"],
    "allow_override": false,
    "local_only": false
  }
}
```

### Submit Override Audit
```http
POST /privacy/audit

Request:
{
  "session_id": "abc123",
  "redaction_count": 0,
  "categories": ["override"],
  "mode": "Override",
  "timestamp": 1730876400
}

Response:
{
  "status": "logged",
  "id": 42
}
```

---

## Implementation Notes

### Technology Stack
- **Framework:** Electron (existing Goose Desktop)
- **UI Library:** React + Tailwind CSS
- **State Management:** Zustand (privacy settings state)
- **API Client:** Axios (calls Controller API)

### Goose Config Integration
Privacy Guard settings in `~/.config/goose/config.yaml`:

```yaml
mcp_servers:
  privacy-guard:
    command: privacy-guard-mcp
    env:
      PRIVACY_MODE: "Hybrid"
      PRIVACY_STRICTNESS: "Strict"
      OLLAMA_URL: "http://localhost:11434"
      CONTROLLER_URL: "http://localhost:8080"
      ENABLE_AUDIT_LOGS: "true"
```

---

## Future Enhancements

### v1.14.0 (Planned)
- Custom regex patterns editor
- Privacy templates (saved override presets)
- Real-time redaction preview

### v1.15.0 (Planned)
- Team sharing of privacy templates
- Compliance reports (monthly summaries)
- ML model selection for NER

---

**End of Specification**

**Next Steps:**
1. Review with UX team
2. Create Figma designs
3. Implement React components
4. Integration testing
5. Release in Goose Desktop v1.13.0
