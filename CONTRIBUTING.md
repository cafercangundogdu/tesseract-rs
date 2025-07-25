# Contributing to tesseract-rs

Thank you for your interest in contributing to tesseract-rs! This document provides guidelines and instructions for contributing.

## Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/) for our commit messages. This leads to more readable messages that are easy to follow when looking through the project history.

### Commit Message Structure

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to our CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Examples

```text
feat: add support for async OCR operations

fix: resolve Windows build issues with HOME environment variable

docs: update README with Windows installation instructions

test: add unit tests for error handling

ci: add commitlint to PR validation
```

### Scope

The scope should be the name of the module affected (as perceived by the person reading the changelog generated from commit messages).

Examples:

- `build`
- `api`
- `error`
- `tests`

### Subject

The subject contains a succinct description of the change:

- Use the imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter
- No dot (.) at the end

### Body

The body should include the motivation for the change and contrast this with previous behavior.

### Footer

The footer should contain any information about Breaking Changes and is also the place to reference GitHub issues that this commit closes.

## Setting Up Commit Hooks

We use Husky to enforce code quality and commit message standards. To set up the hooks:

1. Install Node.js if you haven't already
2. Run `./setup-hooks.sh` or `npm install` in the project root
3. The pre-commit and commit-msg hooks will be automatically installed

The hooks will:
- **Pre-commit**: Check code formatting (rustfmt) and run clippy before allowing commits
- **Commit-msg**: Validate commit messages against our conventional commit format

If pre-commit checks fail:
- Run `cargo fmt` to fix formatting issues
- Run `cargo clippy` to see warnings and fix them

## Pull Request Process

1. Fork the repository and create your branch from `master`
2. If you've added code that should be tested, add tests
3. Ensure the test suite passes with `cargo test`
4. Make sure your code follows the Rust style guide with `cargo fmt`
5. Ensure there are no clippy warnings with `cargo clippy`
6. Update the README.md with details of changes if applicable
7. Create a Pull Request with a clear title and description

## Development Setup

1. Clone the repository
2. Install Rust (latest stable version)
3. Install build dependencies:
   - CMake
   - C++ compiler
   - Node.js (for commit hooks)
4. Run `npm install` to set up commit hooks
5. Run `cargo build` to ensure everything compiles
6. Run `cargo test` to run the test suite

## Code Style

- Follow the official [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md)
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write descriptive variable and function names
- Add comments for complex logic
- Keep functions small and focused

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting a PR
- Add integration tests for significant features
- Test on Windows, macOS, and Linux if possible

## Questions?

Feel free to open an issue if you have any questions or need clarification on anything!
