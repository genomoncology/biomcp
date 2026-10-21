"""Collect source coverage from real CLI/MCP consumers, never Rust unit suites."""
from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
from pathlib import Path

from feature_coverage import coverage_report, inventory


def collect(root: Path, baseline: bool) -> int:
    import tomllib

    output = root / ".cache/feature-coverage"
    output.mkdir(parents=True, exist_ok=True)
    (output / "coverage.json").write_text(json.dumps({"status": "incomplete", "reason": "collection has not finished"}) + "\n")
    links, config, errors = inventory(root)
    if errors:
        raise ValueError("\n".join(errors))
    settings = tomllib.loads((root / "pyproject.toml").read_text()).get("tool", {}).get("biomcp", {}).get("feature-coverage", {})
    floor = settings.get("floor")
    if not baseline and (not isinstance(floor, (int, float)) or not 0 <= floor <= 100):
        raise ValueError("configure the PM-approved outside-in coverage floor first")
    ordinary = os.environ.copy()
    statuses = {}

    def run(name: str, argv: list[str], env: dict | None = None) -> int:
        print(f"{name}: {shlex.join(argv)}", flush=True)
        with (output / f"{name}.log").open("w") as log:
            result = subprocess.run(argv, cwd=root, env=env or ordinary, stdout=log, stderr=subprocess.STDOUT)
        print(f"{name}: exit {result.returncode}; {output / (name + '.log')}", flush=True)
        return result.returncode

    if run("python-prepare", ["make", "sync-python-dev"]):
        return 1
    # The nightly/release job owns its auxiliary executables. Do not replace a
    # different version another lane installed in a shared user directory.
    tooling = output / "tools"
    tool_bin = tooling / "bin"
    tool_env = ordinary | {"UV_TOOL_DIR": str(tooling / "uv"), "UV_TOOL_BIN_DIR": str(tool_bin)}
    ordinary["PATH"] = str(tool_bin) + os.pathsep + ordinary["PATH"]
    version = settings["llvm-cov-version"]
    executable = tool_bin / "cargo-llvm-cov"
    current = subprocess.run([str(executable), "llvm-cov", "--version"], capture_output=True, text=True) if executable.is_file() else None
    if current is None or current.returncode or current.stdout.strip() != "cargo-llvm-cov " + version:
        if run("install-llvm-cov", ["cargo", "install", "cargo-llvm-cov", "--locked", "--version", version, "--root", str(tooling), "--force"]):
            return 1
    if run("llvm-tools", ["rustup", "component", "add", "llvm-tools-preview"]):
        return 1
    workflow = (root / ".github/workflows/ci.yml").read_text()
    mustmatch = re.search(r"^  MUSTMATCH_VERSION: ([0-9.]+)$", workflow, re.M)
    if mustmatch is None:
        raise ValueError("cannot read the configured mustmatch version")
    if run("install-mustmatch", ["uv", "tool", "install", "mustmatch==" + mustmatch[1]], tool_env):
        return 1
    target = output / "target"
    ordinary.update(CARGO_TARGET_DIR=str(target), CARGO_LLVM_COV_TARGET_DIR=str(target),
                    CARGO_LLVM_COV_BUILD_DIR=str(target))
    if run("coverage-clean", ["cargo", "llvm-cov", "clean", "--workspace"]):
        return 1
    show = subprocess.check_output(["cargo", "llvm-cov", "show-env"], cwd=root, env=ordinary, text=True)
    (output / "show-env.txt").write_text(show)
    instrumented = ordinary.copy()
    for line in show.splitlines():
        if "=" not in line:
            continue
        key, value = line.removeprefix("export ").split("=", 1)
        if key.isidentifier() and key.isupper():
            words = shlex.split(value.rstrip(";"))
            instrumented[key] = words[0] if words else ""
    # MCP tests deliberately terminate servers. Continuous counters survive
    # SIGTERM instead of losing every request served before the signal.
    if sys.platform not in {"linux", "darwin"}:
        raise ValueError("outside-in continuous profiling requires Linux or macOS")
    if sys.platform == "linux":
        relocation = "-Cllvm-args=-runtime-counter-relocation"
        wrapper_flags = "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS"
        if wrapper_flags in instrumented:
            instrumented[wrapper_flags] += "\x1f" + relocation
        elif "CARGO_ENCODED_RUSTFLAGS" in instrumented:
            instrumented["CARGO_ENCODED_RUSTFLAGS"] += "\x1f" + relocation
        elif "RUSTFLAGS" in instrumented:
            instrumented["RUSTFLAGS"] += " " + relocation
        else:
            raise ValueError("llvm-cov exported no supported instrumentation flags")
    instrumented["LLVM_PROFILE_FILE"] = str(target / "outside-%c%p.profraw")
    # Compile only binaries/examples and selected integration targets. cfg(test)
    # library/unit artifacts are never built or included as test consumers.
    rust_args = [arg for path in sorted(config["rust"]) for arg in ("--test", Path(path).stem)]
    if run("product-build", ["cargo", "build", "--locked", "--all-features", "--bins", "--examples"], instrumented):
        return 1
    internal = [name for declaration in config["rust"].values()
                for name in declaration.get("internal", {})]
    rust_filter = ["-E", "not test(/^(" + "|".join(re.escape(name) for name in internal) + ")$/)"] if internal else []
    statuses["rust"] = run("rust", ["tools/run-offline", "--", "cargo", "nextest", "run", "--locked", "--all-features", *rust_args, *rust_filter], instrumented)
    consumers = ordinary | {
        "LLVM_PROFILE_FILE": instrumented["LLVM_PROFILE_FILE"],
        "BIOMCP_BIN": str(target / "debug/biomcp"),
        "BIOMCP_ROUTINE_TEST_LANE": "1",
    }
    selected = sorted({link["test"].split("[")[0] for link in links if link["kind"] == "python"
                       and "/live/" not in link["test"]})
    statuses["python"] = run("python", ["tools/run-offline", "--", "uv", "run", "--no-sync", "pytest", *selected,
                                         "-m", "not release_gate and not repo_audit and not long", "--junitxml", str(output / "python.xml")], consumers)
    # The spec runner consumes these artifacts directly and cannot replace the
    # instrumented binary through a nested build. Cargo metadata is ordinary.
    cache = root / ".cache"
    tree = cache / "feature-coverage-tree.txt"
    metadata = cache / "feature-coverage-metadata.json"
    with tree.open("w") as stream:
        subprocess.run(["cargo", "tree", "--locked", "--no-default-features"], cwd=root, env=ordinary, stdout=stream, check=True)
    with metadata.open("w") as stream:
        subprocess.run(["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"], cwd=root, env=ordinary, stdout=stream, check=True)
    exports = {
        "BIOMCP_SPEC_ARTIFACT_MODE": "spec",
        "BIOMCP_SPEC_FEATURE_OFF_BIN": str(target / "debug/biomcp"),
        "BIOMCP_SPEC_MCP_EXAMPLE_BIN": str(target / "debug/examples/rmcp_streamable_http_contract"),
        "BIOMCP_SPEC_CARGO_TREE": str(tree), "BIOMCP_SPEC_CARGO_METADATA": str(metadata),
        "BIOMCP_BIN": str(target / "debug/biomcp"),
    }
    artifact_env = cache / "spec-artifacts.env"
    original = artifact_env.read_bytes() if artifact_env.exists() else None
    try:
        artifact_env.write_text("".join(f"export {key}={shlex.quote(value)}\n" for key, value in exports.items()))
        statuses["spec"] = run("spec", ["tools/run-offline", "--", "bash", "scripts/run-specs.sh", "spec"],
                               consumers | {"BIOMCP_SPEC_ARTIFACTS_PREPARED": "1", "ROUTINE_CARGO_FEATURES": "--no-default-features",
                                            "PATH": str(target / "debug") + os.pathsep + ordinary["PATH"]})
    finally:
        if original is None:
            artifact_env.unlink(missing_ok=True)
        else:
            artifact_env.write_bytes(original)
    (output / "suites.json").write_text(json.dumps(statuses, indent=2) + "\n")
    if any(statuses.values()):
        (output / "coverage.json").write_text(json.dumps({"status": "incomplete", "collections": statuses}, indent=2) + "\n")
        return 1
    lcov = output / "outside.lcov"
    if run("llvm-report", ["cargo", "llvm-cov", "report", "--lcov", "--output-path", str(lcov)], ordinary):
        return 1
    result = coverage_report(root, lcov, statuses, 0 if baseline else floor)
    result["source"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    result["collection"] = {
        "cargo_features": "all-features", "rust_integration_targets": list(config["rust"]),
        "python_selectors": selected, "spec_selection": "scripts/run-specs.sh spec",
        "excluded_execution": "Rust unit suites, live opt-in/ignored tests, other-platform tests and spec pages outside the existing offline routine selection",
    }
    if baseline:
        result.pop("floor")
        result["initial_floor_input"] = result["outside_in_percent"]
    (output / "coverage.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k not in {"unreached", "unobserved_files", "collection"}}, indent=2))
    print(f"Unreached product lines and exact collection selection: {output / 'coverage.json'}")
    return int(result["status"] != "pass")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", action="store_true", help="collect the initial floor input without inventing a threshold")
    args = parser.parse_args()
    try:
        return collect(Path(__file__).resolve().parents[1], args.baseline)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"outside-in collection failed: {error}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
