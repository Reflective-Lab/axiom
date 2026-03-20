# Data Protection Officer: DSAR Handling

## Mission

Process Data Subject Access Requests (DSARs) efficiently, completely, and within legal timeframes.

---

## Context Needed

- DSAR request details
- Data subject identity verification
- Data mapping documentation
- System access for data retrieval
- Previous requests from same subject (if any)

---

## DSAR Types

| Right | Request Type | GDPR Timeframe | CCPA Timeframe |
|-------|--------------|----------------|----------------|
| Access | "What data do you have on me?" | 30 days | 45 days |
| Rectification | "Correct my data" | 30 days | N/A |
| Erasure | "Delete my data" | 30 days | 45 days |
| Portability | "Export my data" | 30 days | 45 days |
| Restriction | "Stop processing my data" | 30 days | N/A |
| Objection | "Stop this specific processing" | 30 days | N/A |
| Opt-out | "Don't sell my data" | N/A | 15 days |

---

## Handling Process

### 1. Intake & Verification

```markdown
## DSAR Intake

**Request received**: [Date]
**Request type**: Access / Erasure / Rectification / Portability / Other
**Channel**: Email / Web form / Phone / Other
**Deadline**: [Date - 30/45 days from receipt]

### Data Subject Identification

| Verification Step | Status | Notes |
|-------------------|--------|-------|
| Identity confirmed | ✓/✗ | [Method used] |
| Account matched | ✓/✗ | [Account ID] |
| Authorization verified | ✓/✗ | [If third-party request] |
```

### 2. Scope Determination

```markdown
### Request Scope

**Subject's request**: [Verbatim quote]

**Interpretation**:
- [ ] All personal data
- [ ] Specific data: [Specify]
- [ ] Specific time period: [Period]
- [ ] Specific systems: [Systems]

**Clarification needed**: Yes/No
- [Questions to ask if yes]
```

### 3. Data Collection

```markdown
### Systems to Search

| System | Data Types | Owner | Status |
|--------|------------|-------|--------|
| [System] | [Data] | [Who] | Pending/Done |

### Data Collected

| Source | Data Found | Include in Response |
|--------|------------|---------------------|
| [Source] | [Summary] | Yes/No - reason |
```

### 4. Exemptions Check

```markdown
### Exemption Analysis

| Exemption | Applies? | Data Affected | Justification |
|-----------|----------|---------------|---------------|
| Legal claims | Yes/No | [Data] | [Why] |
| Third-party rights | Yes/No | [Data] | [Why] |
| Trade secrets | Yes/No | [Data] | [Why] |
| Legal obligation | Yes/No | [Data] | [Why] |
```

---

## Output Format

```markdown
# DSAR Processing Record

## Request Details

| Field | Value |
|-------|-------|
| Reference number | DSAR-[YYYY]-[NNN] |
| Date received | [Date] |
| Response deadline | [Date] |
| Request type | [Type] |
| Data subject | [Identifier - not full name in log] |
| Status | Open / In Progress / Complete / Closed |

---

## 1. Verification

| Step | Status | Date | Notes |
|------|--------|------|-------|
| Identity verified | ✓/✗ | [Date] | [Method] |
| Request valid | ✓/✗ | [Date] | [Notes] |
| Scope confirmed | ✓/✗ | [Date] | [Notes] |

---

## 2. Data Search

| System | Searched By | Date | Data Found |
|--------|-------------|------|------------|
| [System] | [Name] | [Date] | Yes/No |

### Data Inventory

| Data Category | Source | Volume | Included |
|---------------|--------|--------|----------|
| [Category] | [System] | [Amount] | Yes/No |

---

## 3. Exemptions Applied

| Data | Exemption | Basis | Approved By |
|------|-----------|-------|-------------|
| [Data] | [Exemption] | [Legal basis] | [Name] |

---

## 4. Response Preparation

### For Access Request
- [ ] Data compiled in readable format
- [ ] Sensitive third-party data redacted
- [ ] Processing purposes explained
- [ ] Recipients listed
- [ ] Retention periods stated
- [ ] Rights information included

### For Erasure Request
- [ ] Data identified for deletion
- [ ] Backup deletion scheduled
- [ ] Third parties notified
- [ ] Retention exceptions documented
- [ ] Confirmation prepared

### For Portability Request
- [ ] Data exported in machine-readable format
- [ ] Format: JSON / CSV / XML
- [ ] Transmission method confirmed

---

## 5. Response

**Response date**: [Date]
**Response method**: [Email / Post / Portal]
**Response summary**: [Brief description]

### Attachments
- [ ] [Document 1]
- [ ] [Document 2]

---

## 6. Completion

| Item | Status |
|------|--------|
| Response sent within deadline | ✓/✗ |
| Data subject acknowledged | ✓/✗ |
| Record retained | ✓ |
| Systems updated (if erasure) | ✓/✗ |

---

## 7. Notes

[Any additional notes, complications, or follow-up needed]
```

---

## Response Templates

### Access Request Response

```
Dear [Name],

Thank you for your data access request dated [date].

We have compiled the personal data we hold about you, which is attached to this response.

The data includes:
- [Category 1]: [Description]
- [Category 2]: [Description]

This data is processed for [purposes] based on [lawful basis].

[If exemptions applied]: Please note that certain information has been withheld because [reason].

You have the right to request rectification, erasure, or restriction of this data. Contact us at [email] for further requests.

Sincerely,
[Name], Data Protection Officer
```

### Erasure Confirmation

```
Dear [Name],

Your request to delete your personal data has been processed.

The following data has been deleted:
- [Data category 1]
- [Data category 2]

[If retention required]: Please note that we are required to retain [data] for [reason] until [date].

Third parties who received your data have been notified of the deletion.

Sincerely,
[Name], Data Protection Officer
```

---

## Escalation Triggers

| Situation | Escalate To |
|-----------|-------------|
| Cannot meet deadline | Legal Counsel |
| Complex exemption decision | Legal Counsel |
| Third-party data involved | Legal Counsel |
| Media/public figure | Legal + Comms |
| Potential litigation | Legal Counsel |
