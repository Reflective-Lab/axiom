# Priya Chandran — Heartbeat Checklist

Run this every heartbeat cycle.

## 1. Check Assignments

```bash
curl -s "http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/issues?assigneeAgentId=9876bdb0-e398-4478-85f5-65ab4d12824b&status=todo,in_progress,blocked" | python3 -m json.tool
```

## 2. Budget Monitor

```bash
curl -s "http://localhost:3100/api/companies/ad34cffc-6c98-47c9-b629-2aed6f694149/agents" | python3 -c "
import json, sys
agents = json.load(sys.stdin)
total_b, total_s = 0, 0
for a in sorted(agents, key=lambda x: x['spentMonthlyCents']/max(x['budgetMonthlyCents'],1), reverse=True):
    b, s = a['budgetMonthlyCents'], a['spentMonthlyCents']
    total_b += b; total_s += s
    pct = s/b*100 if b > 0 else 0
    flag = ' ⚠️' if pct > 75 else ''
    print(f'{a[\"name\"]:<25} \${s/100:.2f}/\${b/100:.2f} ({pct:.0f}%){flag}')
print(f'---\nTOTAL: \${total_s/100:.2f}/\${total_b/100:.2f} ({total_s/total_b*100:.0f}%)')
"
```

## 3. Work Priority

1. Complete any in_progress issues
2. Check for blocked issues I can unblock
3. Update financial models if new data is available
4. Review agent spend for anomalies
5. Write daily note

## 4. Extraction

After completing work, save key findings to daily note and update deliverables.
