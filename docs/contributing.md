# Contributing to BioMCP

Thank you for your interest in contributing to BioMCP! We welcome contributions from the community. Please take a moment to review these guidelines.

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior. (TODO: Add link to Code of Conduct file if one exists).

## How to Contribute

### Reporting Bugs

- Check the [issue tracker](https://github.com/genomoncology/biomcp/issues) to see if the bug has already been reported.
- If not, open a new issue. Please include:
  - A clear and descriptive title.
  - Steps to reproduce the bug.
  - The expected behavior.
  - The actual behavior (including error messages or logs).
  - Your environment details (Python version, OS, BioMCP version).

### Suggesting Enhancements

- Open an issue on the [issue tracker](https://github.com/genomoncology/biomcp/issues).
- Provide a clear description of the enhancement or feature request.
- Explain the motivation or use case for the proposed change.

### Submitting Pull Requests

1.  **Fork the Repository:** Create your own fork of the `genomoncology/biomcp` repository on GitHub.
2.  **Clone Your Fork:** Clone your fork to your local machine:
    ```bash
    git clone https://github.com/YOUR_USERNAME/biomcp.git
    cd biomcp
    ```
3.  **Create a Branch:** Create a new branch for your changes:
    ```bash
    git checkout -b feature/your-feature-name # or fix/your-bug-fix-name
    ```
4.  **Set Up Development Environment:**
    - Ensure you have Python 3.9+ installed.
    - It's recommended to use a virtual environment:
      ```bash
      python -m venv venv
      source venv/bin/activate # On Windows use `venv\Scripts\activate`
      ```
    - Install BioMCP in editable mode with development dependencies (assuming they are specified, e.g., in `pyproject.toml` or `requirements-dev.txt` - adjust if needed):
      ```bash
      pip install -e ".[dev]" # Or adapt based on project setup
      ```
      If no specific dev extras exist, install pytest and formatters manually:
      ```bash
      pip install -e .
      pip install pytest black ruff # Or other required tools
      ```
5.  **Make Your Changes:** Implement your feature or bug fix.
6.  **Code Formatting and Linting:** Ensure your code adheres to common standards. Run formatters/linters:
    ```bash
    black .
    ruff check . --fix # Or flake8, etc.
    ```
7.  **Run Tests:** Make sure all tests pass. BioMCP uses `pytest` for both unit (TDD) and behavior-driven (BDD) tests.
    ```bash
    pytest
    ```
    Add new tests for your changes if applicable.
8.  **Commit Your Changes:** Use clear and concise commit messages.
    ```bash
    git add .
    git commit -m "feat: Add new variant filtering option" # Or "fix: Correct trial search pagination"
    ```
9.  **Push to Your Fork:**
    ```bash
    git push origin feature/your-feature-name
    ```
10. **Open a Pull Request:** Go to the original `genomoncology/biomcp` repository on GitHub and open a pull request from your branch to the `main` branch. Provide a clear description of your changes in the PR.

## Development Guidelines

- Follow existing code style and patterns.
- Write clear comments where necessary.
- Update documentation (`docs/`) if your changes affect user-facing features or CLI options.
- Ensure tests cover your changes.

Thank you for contributing!
