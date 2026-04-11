# Data Protection Officer: Data Mapping

## Mission

Create and maintain a comprehensive map of personal data flows through Converge—what data we collect, why, where it goes, and how long we keep it.

---

## Context Needed

- System architecture documentation
- Database schemas
- Third-party integrations list
- Current privacy policy
- Existing data inventory (if any)

---

## Mapping Framework

### 1. Data Inventory

For each data element:

```markdown
## Data Element: [Name]

**Category**: Personal / Sensitive / Special Category
**Examples**: [Specific data points]

| Attribute | Value |
|-----------|-------|
| Source | Where does it come from? |
| Lawful basis | Consent / Contract / Legitimate interest / etc. |
| Purpose | Why do we collect it? |
| Storage location | Where is it stored? |
| Retention period | How long do we keep it? |
| Recipients | Who receives it? |
| Cross-border transfer | Does it leave the region? |
```

### 2. Processing Activities

```markdown
## Processing Activity: [Name]

**Description**: What does this activity do?
**Controller/Processor**: Are we controller or processor?
**Data subjects**: Whose data is processed?

| Data Elements | Purpose | Lawful Basis | Retention |
|---------------|---------|--------------|-----------|
| [Element] | [Purpose] | [Basis] | [Period] |
```

### 3. Data Flow Diagram

```
[Source] → [Collection Point] → [Processing] → [Storage] → [Recipients]
                                     ↓
                              [Third Parties]
```

---

## Output Format

```markdown
# Data Mapping Report

## Summary
- **Date**: [date]
- **Scope**: [what was mapped]
- **Data Categories**: [count]
- **Processing Activities**: [count]

---

## 1. Data Inventory

### Personal Data

| Data Element | Category | Source | Purpose | Lawful Basis | Retention |
|--------------|----------|--------|---------|--------------|-----------|
| Name | Identity | User input | Account | Contract | Account lifetime + 3 years |
| Email | Contact | User input | Communication | Contract | Account lifetime + 3 years |
| [etc] | | | | | |

### Sensitive Data

| Data Element | Category | Source | Purpose | Lawful Basis | Safeguards |
|--------------|----------|--------|---------|--------------|------------|
| [If any] | | | | | |

---

## 2. Processing Activities (ROPA)

### Activity: User Account Management

| Attribute | Value |
|-----------|-------|
| Purpose | Manage user accounts and authentication |
| Data subjects | Registered users |
| Data categories | Identity, contact, credentials |
| Recipients | Internal systems, auth provider |
| Transfers | [Countries] |
| Retention | Account lifetime + 3 years |
| Security measures | Encryption, access controls |

### Activity: [Next Activity]
[Repeat structure]

---

## 3. Data Flows

### Inbound Data
| Source | Data | Entry Point | Validation |
|--------|------|-------------|------------|
| [Source] | [Data] | [How it enters] | [Checks] |

### Internal Processing
| Process | Input | Output | Storage |
|---------|-------|--------|---------|
| [Process] | [Data in] | [Data out] | [Where stored] |

### Outbound Data
| Recipient | Data | Purpose | Legal Basis | Safeguards |
|-----------|------|---------|-------------|------------|
| [Recipient] | [Data] | [Why shared] | [Basis] | [Protections] |

---

## 4. Third-Party Processors

| Processor | Service | Data Shared | DPA in Place | Location |
|-----------|---------|-------------|--------------|----------|
| [Vendor] | [Service] | [Data types] | Yes/No | [Country] |

---

## 5. Cross-Border Transfers

| Data | From | To | Transfer Mechanism | Safeguards |
|------|------|-----|-------------------|------------|
| [Data] | [Origin] | [Destination] | SCCs / Adequacy / BCR | [Details] |

---

## 6. Retention Schedule

| Data Category | Retention Period | Basis | Deletion Method |
|---------------|------------------|-------|-----------------|
| [Category] | [Period] | [Why] | [How deleted] |

---

## 7. Gaps Identified

| Gap | Risk | Recommendation | Priority |
|-----|------|----------------|----------|
| [Gap] | H/M/L | [Fix] | P1/P2/P3 |

---

## 8. Action Items

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| [Action] | [Who] | [When] | Open/Done |
```

---

## Mapping Tips

- Start with customer-facing data, then internal
- Trace data through entire lifecycle
- Don't forget logs and backups
- Include automated processing (AI/ML)
- Review third-party integrations carefully
- Update when new features launch
