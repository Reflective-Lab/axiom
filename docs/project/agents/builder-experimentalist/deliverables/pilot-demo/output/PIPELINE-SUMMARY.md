# Pilot Demo Pipeline — Summary

This directory contains a complete simulated pilot data pipeline run.

## Files Generated

| File | Description |
|------|-------------|
| `baseline-report-anon-001.json` | Pre-Converge baseline metrics (Week 0) |
| `pilot-week-1-anon-001.json` | Week 1 pilot telemetry |
| `pilot-week-2-anon-001.json` | Week 2 pilot telemetry |
| `pilot-week-3-anon-001.json` | Week 3 pilot telemetry |
| `pilot-week-4-anon-001.json` | Week 4 pilot telemetry |
| `delta-report-anon-001.json` | Before/after comparison with threshold checks |
| `case-study-anon-001.md` | Anonymized case study (publication-ready draft) |

## Reproducibility

This demo uses fixed data (no randomness). Running the script twice produces identical output.

```bash
bash generate-pilot-demo.sh ./output-1
bash generate-pilot-demo.sh ./output-2
diff -r output-1 output-2  # No differences
```

## What This Proves

1. **Pipeline works end-to-end:** Baseline → telemetry → deltas → case study
2. **Anonymization rules applied:** Customer name replaced, volumes rounded, dates relative
3. **Threshold evaluation automated:** 5/5 thresholds met → "strong_success"
4. **Case study template produces readable output:** Ready for editorial review

## What This Does NOT Prove

- Real customer data flowing through the system
- Converge engine actually running (this is simulated telemetry)
- Statistical significance (noted in case study)
- Integration with real CRM/email systems

## Next Steps

- Connect to converge-experience telemetry exporter (when built)
- Replace static data with live engine output
- Build `scripts/pilot-metrics-aggregate.sh` for real aggregation
- Build `scripts/pilot-anonymize.sh` for real anonymization
