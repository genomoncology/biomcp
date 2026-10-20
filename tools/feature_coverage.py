"""Feature ownership and product-only source coverage for the shipped Rust CLI."""
from __future__ import annotations

import ast
import importlib.util
import json
from functools import lru_cache
import re
from pathlib import Path


FEATURE = re.compile(r"^- ([a-z][a-z0-9-]*): (\S.*)$")


def read_features(root: Path) -> set[str]:
    names: set[str] = set()
    for line in (root / "sdlc/features.md").read_text().splitlines():
        if not line.startswith("- "):
            continue
        match = FEATURE.fullmatch(line)
        if match is None or match[1] in names:
            raise ValueError(f"invalid or repeated feature: {line}")
        names.add(match[1])
    if not names:
        raise ValueError("feature list is empty")
    return names


def python_tests(path: Path) -> list[str]:
    """Read definitions without importing tests or starting their fixtures."""
    def visit(nodes: list[ast.stmt], prefix: str = "") -> list[str]:
        names = []
        for node in nodes:
            if isinstance(node, ast.ClassDef) and node.name.startswith("Test"):
                names.extend(visit(node.body, prefix + node.name + "::"))
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if node.name.startswith("test_"):
                    names.append(prefix + node.name)
        return names
    return visit(ast.parse(path.read_text()).body)


def golden_features(root: Path) -> dict[str, str]:
    tree = ast.parse((root / "tests/golden/test_golden_output.py").read_text())
    cases = next(node.value for node in tree.body if isinstance(node, ast.Assign)
                 and any(isinstance(n, ast.Name) and n.id == "CASES" for n in node.targets))
    links = {}
    for case in cases.elts:
        argv = list(ast.literal_eval(case.args[1]))
        if argv[0] == "--disable-sources":
            argv = argv[2:]
        feature = argv[1] if argv[0] in {"get", "search"} else argv[0]
        feature = "search-all" if feature == "all" else feature
        formats = next((ast.literal_eval(k.value) for k in case.keywords
                        if k.arg == "formats"), ("json", "markdown"))
        for fmt in formats:
            links[f"test_golden_output[{ast.literal_eval(case.args[0])}-{fmt}]"] = feature
    return links


def inventory(root: Path) -> tuple[list[dict[str, str]], dict, list[str]]:
    names = read_features(root)
    config = json.loads((root / "tests/feature-links.json").read_text())
    links = []
    errors = []
    seen = set()
    for kind in ("python", "rust", "spec"):
        for relative, declaration in config[kind].items():
            path = root / relative
            seen.add(relative)
            if not path.is_file():
                errors.append(f"missing test file {relative}")
                continue
            if kind == "python":
                tests = python_tests(path)
            elif kind == "rust":
                # Names identify the existing integration test, not every helper fn.
                tests = re.findall(r"#\[(?:test|tokio::test[^\]]*)\](?:\s*#\[[^\]]*\])*\s*"
                                   r"(?:async\s+)?fn\s+(\w+)", path.read_text())
            else:
                tests = [f"block-{i}" for i, _ in enumerate(
                    re.finditer(r"^```(?:bash|sh)\b", path.read_text(), re.M), 1)]
            overrides = declaration.get("tests", {})
            if kind != "spec":
                for missing in overrides.keys() - set(tests):
                    errors.append(f"missing test {relative}::{missing}")
            if not tests:
                errors.append(f"{relative} has no test")
            for test in tests:
                owner = overrides.get(test, declaration.get("feature"))
                if owner is None:
                    if test not in declaration.get("internal", {}):
                        errors.append(f"unclassified test {relative}::{test}")
                    continue
                if owner == "@golden":
                    for instance, feature in golden_features(root).items():
                        links.append({"kind": kind, "test": f"{relative}::{instance}",
                                      "feature": feature})
                else:
                    links.append({"kind": kind, "test": f"{relative}::{test}",
                                  "feature": owner})
    for relative, reason in config["internal"].items():
        if not reason or not (root / relative).is_file():
            errors.append(f"invalid internal-test classification {relative}")
        seen.add(relative)
    # New test modules require an explicit surface or internal-test classification.
    discovered = {str(p.relative_to(root)) for p in (root / "tests").rglob("test_*.py")}
    discovered.update(str(p.relative_to(root)) for p in (root / "tests").glob("*.rs"))
    for relative in sorted(discovered - seen):
        errors.append(f"unclassified test module {relative}")
    owners = {link["feature"] for link in links}
    errors.extend(f"unknown feature {feature}" for feature in sorted(owners - names))
    errors.extend(f"{feature} has no outside-in test" for feature in sorted(names - owners))
    return links, config, errors


@lru_cache
def rust_lexer(root: Path):
    # Reuse the repository's Rust lexer; comments, raw strings and nested block
    # comments cannot change the extent of a cfg(test) item.
    tool = root / "tools/check-quality-ratchet.py"
    if not tool.is_file():
        tool = Path(__file__).with_name("check-quality-ratchet.py")
    spec = importlib.util.spec_from_file_location("semantic_contracts", tool)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module._mask_rust_non_code


def inline_tests(root: Path, source: Path) -> set[int]:
    text = source.read_text()
    masked = rust_lexer(root)(text)
    excluded = set()
    for match in re.finditer(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", masked):
        start = match.start()
        brace = masked.find("{", match.end())
        semicolon = masked.find(";", match.end())
        if semicolon >= 0 and (brace < 0 or semicolon < brace):
            end = semicolon + 1
        elif brace >= 0:
            depth, end = 1, brace + 1
            while end < len(masked) and depth:
                depth += (masked[end] == "{") - (masked[end] == "}")
                end += 1
        else:
            raise ValueError(f"cannot locate cfg(test) item in {source}")
        excluded.update(range(text.count("\n", 0, start) + 1,
                              text.count("\n", 0, end) + 2))
    return excluded


def coverage_report(root: Path, lcov: Path, statuses: dict, floor: float) -> dict:
    incomplete = {name: statuses.get(name, "missing") for name in ("rust", "python", "spec")
                  if statuses.get(name) != 0}
    if incomplete:
        return {"status": "incomplete", "collections": incomplete}
    product = {}
    excluded = {}
    relative = None
    for line in lcov.read_text().splitlines():
        if line.startswith("SF:"):
            path = Path(line[3:])
            if not path.is_absolute():
                path = root / path
            try:
                candidate = path.resolve().relative_to(root.resolve())
            except ValueError:
                relative = None
                continue
            if (candidate.parts[0] != "src" or candidate.suffix != ".rs"
                    or any(p in {"tests", "test_support", "generated"} for p in candidate.parts)
                    or re.search(r"(^test[s]?_|_test[s]?\.rs$|^test[s]?\.rs$)", candidate.name)):
                relative = None
                continue
            relative = candidate.as_posix()
            if relative not in excluded:
                excluded[relative] = inline_tests(root, path)
            product.setdefault(relative, {})
        elif relative and line.startswith("DA:"):
            number, hits, *_ = line[3:].split(",")
            number, hits = int(number), int(hits)
            if number not in excluded[relative]:
                product[relative][number] = max(hits, product[relative].get(number, 0))
    total = sum(map(len, product.values()))
    if not total:
        raise ValueError("coverage has no instrumented product lines")
    reached = sum(sum(value > 0 for value in lines.values()) for lines in product.values())
    percentage = 100 * reached / total
    return {"status": "pass" if percentage >= floor else "fail", "floor": floor,
            "denominator": "instrumented src/**/*.rs product lines; excludes test files, inline cfg(test) items, test_support and generated code",
            "product_lines": total, "reached_lines": reached,
            "outside_in_percent": percentage,
            "unobserved_files": sorted(str(p.relative_to(root)) for p in (root / "src").rglob("*.rs")
                                       if str(p.relative_to(root)) not in product
                                       and not any(part in {"tests", "test_support", "generated"} for part in p.relative_to(root).parts)
                                       and not re.search(r"(^test[s]?_|_test[s]?\.rs$|^test[s]?\.rs$)", p.name)),
            "unreached": {path: [number for number, hits in lines.items() if hits == 0]
                          for path, lines in product.items() if any(h == 0 for h in lines.values())}}
