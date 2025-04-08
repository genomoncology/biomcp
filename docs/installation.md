# Installation Guide

This guide provides instructions for installing BioMCP.

## Prerequisites

- **Python:** BioMCP requires Python 3.9 or later. You can check your Python version by running `python --version` or `python3 --version`.
- **pip:** Python's package installer is required. It usually comes bundled with Python. You can check if pip is installed by running `pip --version` or `pip3 --version`.

## Installation Methods

### Method 1: Install from PyPI (Recommended)

This is the standard and recommended way to install BioMCP for most users.

1.  Open your terminal or command prompt.
2.  Run the following command:

    ```bash
    pip install biomcp-python
    ```

    Or, depending on your system configuration:

    ```bash
    pip3 install biomcp-python
    ```

    You can also use other package managers like `uv`:

    ```bash
    uv pip install biomcp-python
    ```

    This command downloads BioMCP and its required dependencies from the Python Package Index (PyPI) and installs them.

### Method 2: Install from Source (for Development)

If you want to contribute to BioMCP development or install the latest unreleased version, you can install it directly from the source code.

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/genomoncology/biomcp.git
    cd biomcp
    ```

2.  **Install in editable mode:** Using `pip` in editable mode (`-e`) allows you to make changes to the code without reinstalling.
    ```bash
    pip install -e .
    ```
    Or, using `pip3`:
    ```bash
    pip3 install -e .
    ```
    _(Note: If the project uses a different build system like Poetry or Hatch, follow its specific instructions for installing in editable mode.)_

## Verification

After installation, you can verify that BioMCP is installed correctly by running:

```bash
biomcp --help
```

This should display the main help message listing the available commands (trial, article, variant, run).

You can also check the help for a specific command:

```bash
biomcp variant --help
```

## Dependencies

BioMCP relies on several external Python libraries, such as typer, httpx, pydantic, and diskcache. These dependencies are automatically handled when you install BioMCP using pip.
