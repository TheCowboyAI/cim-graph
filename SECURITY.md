# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability within cim-graph, please follow these steps:

1. **DO NOT** disclose the vulnerability publicly until it has been addressed.
2. Email the details to security@thecowboy.ai with:
   - A description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your vulnerability report within 48 hours.
- **Assessment**: We will assess the vulnerability and determine its severity within 7 days.
- **Resolution**: We aim to release a fix within 30 days of the initial report, depending on complexity.
- **Disclosure**: Once the vulnerability is fixed, we will work with you on responsible disclosure.

## Security Best Practices

When using cim-graph in your projects:

1. **Keep Dependencies Updated**: Regularly update to the latest version of cim-graph.
2. **Review Graph Data**: Be cautious when deserializing graph data from untrusted sources.
3. **Validate Input**: Always validate node and edge data before adding to graphs.
4. **Use Type Safety**: Leverage Rust's type system to prevent common security issues.

## Security Features

cim-graph includes several security-focused features:

- **Memory Safety**: Built with Rust's memory safety guarantees
- **Type Safety**: Strong typing prevents many common vulnerabilities
- **No Unsafe Code**: The core library avoids unsafe Rust code
- **Input Validation**: Built-in validation for graph operations

## Dependency Management

We regularly audit our dependencies for known vulnerabilities using:
- `cargo audit`
- GitHub's Dependabot alerts
- Manual review of critical dependencies

## Contact

For security-related inquiries, contact: security@thecowboy.ai

For general questions, use [GitHub Discussions](https://github.com/thecowboyai/cim-graph/discussions).