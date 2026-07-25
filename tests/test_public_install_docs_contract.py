from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
INSTALLER_COMMAND = "curl -fsSL https://biomcp.org/install.sh | bash"
UV_INSTALL_COMMAND = "uv tool install biomcp-cli"
PIP_INSTALL_COMMAND = "pip install biomcp-cli"


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _first_bash_code_block_after(text: str, marker: str) -> str:
    start = text.index(marker)
    fence_start = text.index("```bash", start)
    block_start = text.index("\n", fence_start) + 1
    block_end = text.index("\n```", block_start)
    return text[block_start:block_end]


def _assert_install_order(text: str) -> None:
    assert INSTALLER_COMMAND in text
    assert UV_INSTALL_COMMAND in text
    assert PIP_INSTALL_COMMAND in text
    assert text.index(INSTALLER_COMMAND) < text.index(UV_INSTALL_COMMAND)
    assert text.index(UV_INSTALL_COMMAND) < text.index(PIP_INSTALL_COMMAND)


def test_installation_doc_covers_binary_first_and_pypi_command_contract() -> None:
    installation = _read("docs/getting-started/installation.md")

    assert "## Option 1: Installer script" in installation
    assert "## Option 2: PyPI package" in installation
    assert "## Option 3: Source build" in installation
    assert installation.index("## Option 1: Installer script") < installation.index(
        "## Option 2: PyPI package"
    )

    installer_block = _markdown_section_block(
        installation,
        "## Option 1: Installer script",
        "\n## Option 2: PyPI package",
    )
    assert INSTALLER_COMMAND in installer_block
    assert "bash -s -- --version 0.8.0" in installer_block
    assert "fails closed" in installer_block
    assert "sha256sum" in installer_block
    assert "shasum -a 256" in installer_block
    assert "openssl dgst -sha256" in installer_block
    assert "biomcp --version" in installer_block

    pypi_block = _markdown_section_block(
        installation,
        "## Option 2: PyPI package",
        "\n## Option 3: Source build",
    )
    assert UV_INSTALL_COMMAND in pypi_block
    assert PIP_INSTALL_COMMAND in pypi_block
    assert "Install the `biomcp-cli` package, then use the `biomcp` command" in pypi_block
    assert "biomcp --version" in pypi_block


def test_installation_doc_source_build_uses_canonical_make_install_path() -> None:
    installation = _read("docs/getting-started/installation.md")

    source_block = _markdown_section_block(
        installation,
        "## Option 3: Source build",
        "\n## Post-install smoke checks",
    )

    assert "make install" in source_block
    assert '"$HOME/.local/bin/biomcp" --version' in source_block
    assert "cargo install --path ." not in source_block


def test_readme_lists_binary_install_before_pypi_tool_install() -> None:
    readme = _read("README.md")

    install_block = _markdown_section_block(
        readme,
        "## Installation",
        "\n## Quick start",
    )

    assert "### Binary install" in install_block
    assert "### PyPI tool install" in install_block
    assert install_block.index("### Binary install") < install_block.index(
        "### PyPI tool install"
    )
    _assert_install_order(install_block)


def test_docs_index_lists_binary_install_before_pypi_install() -> None:
    docs_index = _read("docs/index.md")

    assert "### Binary install" in docs_index
    assert "### PyPI tool install" in docs_index
    assert docs_index.index("### Binary install") < docs_index.index(
        "### PyPI tool install"
    )
    _assert_install_order(docs_index)
    assert "Install the `biomcp-cli` package, then use `biomcp`" in docs_index


def test_quick_reference_install_block_covers_supported_public_paths() -> None:
    quick_reference = _read("docs/reference/quick-reference.md")

    install_block = _markdown_section_block(
        quick_reference,
        "## Install",
        "\n## Core command grammar",
    )

    assert "**Binary installer (recommended):**" in install_block
    assert "**PyPI tool install:**" in install_block
    assert install_block.index("**Binary installer (recommended):**") < install_block.index(
        "**PyPI tool install:**"
    )
    _assert_install_order(install_block)
    assert "Install the `biomcp-cli` package, then use the `biomcp` command" in install_block
    assert "../getting-started/installation.md" in install_block


def test_ticketed_blog_install_blocks_put_curl_before_uv_and_pip() -> None:
    cases = [
        ("docs/blog/cbioportal-study-analytics.md", "## Try it"),
        ("docs/blog/biomcp-charts.md", "## Try it"),
        ("docs/blog/biomcp-pubmed-articles.md", "## Try it"),
        ("docs/blog/skillbench-biomcp-skills.md", "## Try it"),
        ("docs/blog/kras-g12c-treatment-landscape.md", "## Try it"),
        (
            "docs/blog/we-deleted-35-tools.md",
            "install and first query in under 60 seconds:",
        ),
        ("docs/blog/we-deleted-35-tools.md", "## Try it"),
    ]

    for path, marker in cases:
        block = _first_bash_code_block_after(_read(path), marker)
        _assert_install_order(block)
