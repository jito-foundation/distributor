# Distributor Smoke Test

A small end-to-end script that creates a Merkle distributor, funds its vault,
and exercises `new_claim` along with admin flows. The script targets a running
local validator.

## Run

```bash
# Start a validator separately
# solana-test-validator -r --quiet &

pnpm ts-node scripts/distributor_smoke.ts
```

Adjust the vesting timestamps in `distributor_smoke.ts` (the variables `startTs`
and `endTs`) for quicker runs.
