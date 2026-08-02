---
change: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
artifact: research
---

# Research

## Observed evidence

- Candidate `b117382f08bf5f56965a2a1cf394b2d690662b8d` had a successful official GitHub Actions
  `SpecSync trusted policy` check with the expected external revision binding.
- GitHub returned `https://github.com/CorvidLabs/spec-sync/runs/<check-id>` despite the publisher
  supplying an Actions workflow-run URL.
- The successful `pull_request_target` workflow run remained queryable with the exact candidate SHA,
  repository, workflow path, event, conclusion, and PR association.

## Conclusion

The verifier must treat the check details URL as presentation metadata and independently authenticate
the workflow run through GitHub's Actions API. Ambiguous or mismatched runs remain failures.
