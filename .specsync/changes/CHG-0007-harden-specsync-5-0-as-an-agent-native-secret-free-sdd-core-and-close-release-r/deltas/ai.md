## REMOVED

### REQUIREMENT REQ-ai-001
Embedded AI compatibility paths are removed from the 5.0 core. Coding agents own inference and credentials outside the SpecSync trust boundary.

## ADDED

### REQUIREMENT REQ-ai-002
The deprecated AI module SHALL remain only as historical documentation and SHALL expose no production source file or runtime behavior.

Acceptance Criteria
- No embedded LLM client, API key, provider, model, base URL, source upload, or shell escape remains.
- Native coding-agent and MCP integration remains available outside this retired module.
