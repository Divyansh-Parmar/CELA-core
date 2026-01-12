# Security Policy

## Supported Versions
Only the latest stable version of CELA is supported with security updates.

## Reporting a Vulnerability
If you discover a security vulnerability, please do **not** open a public issue.
Instead, please send an email to the project maintainers (or use GitHub Security Advisories if enabled).

## Scope
We are particularly interested in:
- **Remote Code Execution (RCE):** Malicious models or prompts executing code.
- **Resource Exhaustion:** Bypassing hard limits (tokens/time) to cause DoS.
- **Privacy Leaks:** Memory layer leakage across sessions (note: current memory is single-tenant file-based).

## Safe Model Loading
**Warning:** Only load models (`.gguf`) from trusted sources. While GGUF is generally safer than pickle-based formats, parsing complex binary files always carries some risk.
