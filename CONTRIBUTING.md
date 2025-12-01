# Contributing to Reminex

First off, thank you for considering contributing to Reminex! It's people like you that make Reminex such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples** (command line arguments, file structures, etc.)
- **Describe the behavior you observed and what you expected**
- **Include your OS, Rust version, and Reminex version**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description of the suggested enhancement**
- **Explain why this enhancement would be useful**
- **List some examples of how it would be used**

### Pull Requests

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. Ensure the test suite passes (`cargo test`)
4. Make sure your code lints (`cargo clippy`)
5. Format your code (`cargo fmt`)
6. Issue that pull request!

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/reminex.git
cd reminex

# Build the project
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## Coding Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Write clear commit messages
- Add tests for new features
- Update documentation as needed

## Testing

- Write unit tests for new functions
- Write integration tests for new features
- Ensure all tests pass before submitting PR
- Aim for high code coverage

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Documentation

- Add rustdoc comments for public APIs
- Update README.md if adding new features
- Include examples in documentation
- Keep inline comments concise and clear

## Git Workflow

1. Create a feature branch: `git checkout -b feature/amazing-feature`
2. Commit your changes: `git commit -m 'Add some amazing feature'`
3. Push to the branch: `git push origin feature/amazing-feature`
4. Open a Pull Request

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

## AI-Assisted Development

This project embraces AI-assisted development. If you use AI tools (GitHub Copilot, Claude, etc.) to help with your contribution:

- Review all AI-generated code carefully
- Ensure the code meets our quality standards
- Add appropriate tests
- Verify that the implementation is correct and efficient

## Questions?

Feel free to open an issue with your question or join our discussions!

## Attribution

This Contributing Guide is adapted from open-source contribution guidelines.
