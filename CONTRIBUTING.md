# Contributing to CSV Converter

Thank you for considering contributing to CSV Converter! We welcome contributions from everyone.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:
- A clear description of the problem
- Steps to reproduce the issue
- Expected vs actual behavior
- Sample CSV file (if applicable)
- Your environment (OS, Rust version)

### Suggesting Features

Feature requests are welcome! Please open an issue describing:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered
- Examples of how it would work

### Pull Requests

1. **Fork the repository** and create your branch from `master`
2. **Make your changes** with clear, descriptive commits
3. **Add tests** for any new functionality
4. **Ensure all tests pass**: `cargo test`
5. **Run formatting**: `cargo fmt`
6. **Run linting**: `cargo clippy`
7. **Update documentation** if needed (README, code comments)
8. **Submit your pull request**

### Development Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/csv-converter.git
cd csv-converter

# Build the project
cargo build

# Run tests
cargo test

# Run the converter
cargo run -- --input test.csv
```

### Code Style

- Follow Rust standard style guidelines
- Use `cargo fmt` to format code
- Run `cargo clippy` and address warnings
- Write clear commit messages
- Add comments for complex logic
- Keep functions focused and testable

### Testing

- Add unit tests for new functions in `src/lib.rs`
- Add integration tests in `tests/` for end-to-end features
- Ensure all tests pass before submitting PR
- Include test cases for edge cases

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --lib
cargo test --test format_detection_tests
cargo test --test integration_tests
```

### Documentation

- Update README.md for user-facing changes
- Add inline code documentation for complex logic
- Include examples in function documentation
- Update help text if adding CLI options

## Code of Conduct

Please note that this project is released with a [Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## Questions?

Feel free to open an issue for any questions about contributing!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
