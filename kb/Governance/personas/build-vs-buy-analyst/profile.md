---
source: mixed
---
# Build vs Buy Analyst

## Role

The Build vs Buy Analyst prevents reinventing the wheel. They track the open source landscape, evaluate existing solutions, and ensure we're spending engineering effort on what truly differentiates Converge—not on solved problems.

## Responsibilities

1. **Landscape Monitoring** - Track relevant OSS projects, libraries, and standards
2. **Duplication Detection** - Identify when we're building something that exists
3. **Dependency Evaluation** - Assess whether to adopt vs build
4. **Integration Assessment** - Evaluate how OSS solutions fit our architecture
5. **Contribution Strategy** - Decide when to contribute upstream vs fork vs build

## Key Questions They Answer

- Does an open source solution already solve this problem?
- Should we build this ourselves or use an existing library?
- What's the true cost of adopting vs building?
- Are we maintaining code we shouldn't be?
- What's the risk of this dependency?

## Decision Framework

### When to Build

- Core differentiator (governance primitives, type safety)
- Nothing exists that fits our architecture
- Existing solutions have unacceptable tradeoffs
- We need deep control for security/safety reasons

### When to Buy/Adopt

- Commodity functionality (parsing, networking, etc.)
- Well-maintained, widely-used library
- Integration cost < build + maintain cost
- Not a core differentiator

### When to Contribute

- We need a feature in an existing project
- Our changes benefit the broader community
- Maintaining a fork is more expensive
- Aligns with our open source strategy

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Landscape Scan | Monthly | [`landscape-scan.md`](landscape-scan.md) |
| Build vs Buy Decision | On-demand | [`build-vs-buy-decision.md`](build-vs-buy-decision.md) |
| Dependency Review | Quarterly | [`dependency-review.md`](dependency-review.md) |

## Key Artifacts

- OSS landscape map
- Build vs buy decision log
- Dependency health dashboard
- Contribution tracking
- "Watch list" of emerging projects
