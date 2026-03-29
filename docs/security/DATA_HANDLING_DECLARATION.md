# Data Handling Declaration

## Purpose

This document states what the project is intended to process, what should be
treated as high risk, and what is out of scope for the open source repository.

## Designed To Handle

The project is designed to orchestrate agent workflows and related operational
data such as:

- prompts, instructions, and workflow configuration
- model inputs and outputs
- structured business records supplied by deployers
- audit events, traces, and execution metadata
- policy decisions and provenance records

## May Handle If The Deployer Explicitly Chooses To

The software can technically be used with regulated or sensitive data, but that
does not mean this repository alone makes such use compliant. Examples:

- personal data
- confidential internal business data
- customer support content
- contract or procurement records
- enterprise knowledge-base content

Use with these categories requires deployer-owned controls for access,
retention, deletion, encryption, logging, vendor review, and legal basis.

## Not Declared As Supported By Default

This repository does not currently declare out-of-the-box support or compliance
attestation for:

- protected health information under HIPAA
- cardholder data environments under PCI DSS
- export-controlled or classified information
- biometric identifiers requiring special legal handling
- highly sensitive government data

If a deployer wants to process these categories, they must perform a dedicated
review, add environment-specific controls, and obtain the required legal and
compliance approvals.

## Data We Recommend Avoiding

Unless there is a documented business need and approved control set, do not
place the following into prompts, logs, or long-lived context:

- raw credentials, API keys, session tokens, or private keys
- full payment card numbers or card verification values
- full medical records
- national ID numbers or passport numbers
- highly sensitive HR or disciplinary records
- secrets copied from production systems for debugging convenience

## Logging and Retention Position

- logs should be treated as potentially sensitive
- prompts and model outputs may contain customer data and should not be assumed
  safe for unrestricted retention
- retention schedules must be defined by deployers and contracts, not inferred
  from repository defaults

## Controller and Processor Boundaries

This open source project is software, not a legal entity acting as a data
controller or processor by itself. In a real deployment:

- the deploying organization determines purposes and means of processing
- hosted vendors and model providers may act as subprocessors
- customer contracts and privacy terms must define actual roles

## Declaration Summary

Converge is intended to support enterprise agent workflows with explicit policy,
traceability, and human oversight. It is not a blanket declaration that any
data type is appropriate to process without deployment-specific controls.
