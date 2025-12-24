# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Features

GameVault includes several security features by default:

### Network Security
- **Localhost binding** - Server binds to `127.0.0.1` by default
- **CORS restrictions** - Only localhost origins allowed by default
- **No external exposure** - Must explicitly set `HOST=0.0.0.0` to expose

### Authentication
- **Optional API key** - Set `API_KEY` to protect sensitive endpoints
- **Protected endpoints** - `/scan`, `/enrich`, `/export` require auth when enabled

### Data Protection
- **Path hiding** - Local file paths hidden from API responses
- **Input validation** - Search queries limited to 200 characters
- **Error sanitization** - Database errors not exposed to clients
- **SQL injection prevention** - All queries use parameterized statements

### Build Security
- **Dependency auditing** - Weekly security scans via GitHub Actions
- **SBOM generation** - Software Bill of Materials for supply chain security
- **Container scanning** - Docker images scanned with Trivy
- **Secret detection** - TruffleHog scans for leaked credentials

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability:

### Do NOT
- Open a public GitHub issue
- Disclose publicly before we've had a chance to fix it

### Do
1. **Email**: Send details to security@gamevault.example (replace with actual email)
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

### Response Timeline
- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Fix timeline**: Depends on severity
  - Critical: Within 24 hours
  - High: Within 7 days
  - Medium: Within 30 days
  - Low: Next regular release

### Recognition
We appreciate responsible disclosure and will:
- Credit you in the release notes (if desired)
- Keep you updated on the fix progress
- Not take legal action for good-faith research

## Security Best Practices

### For Users

1. **Keep Updated**
   - Use the latest version
   - Enable Dependabot alerts on your fork

2. **Network Security**
   - Don't expose to the internet without authentication
   - Use a reverse proxy (nginx, Caddy) with HTTPS for remote access
   - Set `API_KEY` when exposing to network

3. **Configuration**
   ```toml
   [server]
   bind_address = "127.0.0.1"  # Keep this unless you need network access
   ```

4. **Docker**
   ```yaml
   # Mount games as read-only
   volumes:
     - /games:/games:ro
   ```

### For Contributors

1. **Dependencies**
   - Run `cargo audit` before submitting PRs
   - Run `npm audit` for frontend changes
   - Don't add unnecessary dependencies

2. **Code Review**
   - All changes require review
   - Security-sensitive files require security team review
   - Follow OWASP guidelines

3. **Secrets**
   - Never commit secrets
   - Use environment variables
   - Pre-commit hooks check for secrets

## Security Checklist

For maintainers when releasing:

- [ ] All dependencies audited
- [ ] No high/critical vulnerabilities
- [ ] Docker image scanned
- [ ] SBOM generated
- [ ] Checksums provided
- [ ] Security-sensitive changes reviewed

## Compliance

GameVault follows:
- OWASP Top 10 guidelines
- Secure coding practices for Rust
- React security best practices
- Container security best practices

## Contact

- Security issues: security@gamevault.example
- General questions: Open a GitHub discussion
