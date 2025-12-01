# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please do the following:

1. **Do NOT** open a public issue
2. Email the details to [your-email@example.com] (replace with actual email)
3. Include as much information as possible:
   - Type of vulnerability
   - Full paths of source file(s) affected
   - Step-by-step instructions to reproduce
   - Proof-of-concept or exploit code (if possible)
   - Impact of the issue

## What to Expect

- You will receive a response within 48 hours
- We will confirm the vulnerability and determine its severity
- We will release a fix as soon as possible (depending on complexity)
- We will credit you in the release notes (unless you prefer to remain anonymous)

## Security Best Practices

When using Reminex:

- Keep your Rust toolchain updated
- Regularly update Reminex to the latest version
- Be cautious when indexing untrusted directories
- Review database file permissions
- Don't share database files containing sensitive path information

## Known Security Considerations

- Database files are unencrypted SQLite files
- File paths are stored in plain text
- No authentication mechanism for database access
- Ensure proper file system permissions on database files

Thank you for helping keep Reminex and its users safe!
