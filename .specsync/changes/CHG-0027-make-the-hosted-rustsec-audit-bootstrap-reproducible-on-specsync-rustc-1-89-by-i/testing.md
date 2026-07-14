---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: testing
---

# Testing

- Parse `.github/workflows/ci.yml` as YAML.
- Run `fledge run audit` to evaluate the repository lockfile with the committed policy.
- Run `specsync change verify` and strict lifecycle validation at 100% coverage.
- Require the hosted `audit` job to install the locked tool and pass before merge.
