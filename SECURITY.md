# Security Policy

## Supported Versions

This repository contains educational projects.  Security fixes are applied to
the **latest commit on `main`** only.  Older commits or released snapshots are
not back-patched.

| Project | Supported |
| ------- | --------- |
| `rust_file_encrypt` (latest `main`) | ✅ Yes |
| `data_science` notebooks | ✅ Yes |
| Any pinned/archived version | ❌ No |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Use GitHub's built-in **private security advisory** feature to report issues
confidentially:

1. Go to the **Security** tab of this repository.
2. Click **"Report a vulnerability"**.
3. Fill in the details (description, reproduction steps, potential impact, and
   any suggested fix).

You can also reach the maintainer directly through the contact information
listed on the GitHub profile.

### What to Include

A useful report includes:

- A clear description of the vulnerability.
- Steps to reproduce the issue.
- The affected component(s) and version/commit.
- The potential impact (e.g., data exposure, privilege escalation).
- Any suggested mitigation or patch (optional but welcome).

## Response Timeline

| Action | Target time |
| ------ | ----------- |
| Initial acknowledgement | Within **3 business days** |
| Triage and severity assessment | Within **7 days** |
| Patch or mitigation published | Within **30 days** for critical/high; best effort for lower severity |
| Public disclosure (CVE if applicable) | After patch is available, coordinated with reporter |

## Disclosure Policy

This project follows **coordinated disclosure**:

- Vulnerabilities are kept private until a fix is ready or the 90-day
  disclosure deadline is reached (whichever comes first).
- The reporter will be credited in the security advisory unless they prefer to
  remain anonymous.
- If a vulnerability is determined to be out of scope or not reproducible, the
  reporter will be notified with an explanation.

## Security Best Practices for Contributors

- Never commit secrets, API keys, or credentials to the repository.
- Keep dependencies up-to-date; Dependabot is configured to open automated
  update PRs.
- Run `cargo audit` locally before submitting Rust changes:
  ```bash
  cargo install cargo-audit
  cargo audit
  ```
- Review the automated security scan results in the **Actions** tab before
  merging any pull request.
