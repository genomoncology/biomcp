"""Feature ownership and product-only source coverage for the shipped Rust CLI."""
from __future__ import annotations

import ast
import importlib.util
import json
import xml.etree.ElementTree as ET
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



def golden_call_feature(root: Path, path: Path, test: str) -> str:
    owners = {instance.partition("[")[2].removesuffix("]").rsplit("-", 1)[0]: feature
              for instance, feature in golden_features(root).items()}
    function = next(node for node in ast.parse(path.read_text()).body
                    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == test)
    cases = {node.args[1].value for node in ast.walk(function) if isinstance(node, ast.Call)
             and isinstance(node.func, ast.Name) and node.func.id == "_run"
             and len(node.args) > 1 and isinstance(node.args[1], ast.Constant)}
    features = {owners[case] for case in cases}
    if len(features) != 1:
        raise ValueError(f"{path.name}::{test} needs one explicit owning feature")
    return features.pop()


def rust_tests(root: Path, path: Path, prefix: str = "", ancestors: tuple[Path, ...] = ()) -> list[str]:
    """Follow integration-test module declarations, retaining nextest names."""
    path = path.resolve()
    if path in ancestors:
        raise ValueError(f"cyclic Rust test module {path}")
    text = path.read_text()
    masked = rust_lexer(root)(text)
    if re.search(r"\bmod\s+\w+\s*\{", masked):
        raise ValueError(f"inline Rust integration-test modules need qualified inventory support: {path}")
    declarations = re.finditer(
        r"#\[(?:test|tokio::test[^\]]*)\](?:\s*#\[[^\]]*\])*\s*"
        r"(?:async\s+)?fn\s+(\w+)", masked)
    names = [prefix + match[1] for match in declarations]
    # Existing integration targets use external modules. Read the path string
    # from the original text because the Rust lexer masks string contents.
    for match in re.finditer(r"\bmod\s+(\w+)\s*;", masked):
        attributes = text[:match.start()]
        explicit = re.search(r'#\[path\s*=\s*"([^"]+)"\]\s*$', attributes)
        if explicit:
            child = path.parent / explicit[1]
        else:
            directory = path.parent if not prefix or path.name == "mod.rs" else path.parent / path.stem
            candidates = [directory / (match[1] + ".rs"), directory / match[1] / "mod.rs"]
            existing = [candidate for candidate in candidates if candidate.is_file()]
            if len(existing) != 1:
                raise ValueError(f"cannot resolve Rust test module {path}::{match[1]}")
            child = existing[0]
        names.extend(rust_tests(root, child, prefix + match[1] + "::", (*ancestors, path)))
    return names


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
                tests = rust_tests(root, path)
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
                if owner == "@golden-calls":
                    owner = golden_call_feature(root, path, test)
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
    discovered.update(str(p.relative_to(root)) for p in (root / "spec").rglob("*.md")
                      if re.search(r"^```(?:bash|sh)\b", p.read_text(), re.M))
    for relative in sorted(discovered - seen):
        errors.append(f"unclassified test module {relative}")
    owners = {link["feature"] for link in links}
    errors.extend(f"unknown feature {feature}" for feature in sorted(owners - names))
    errors.extend(f"{feature} has no outside-in test" for feature in sorted(names - owners))
    case_owners = {link["test"]: link["feature"] for link in links}
    for feature, case in config.get("accounting_cases", {}).items():
        if case_owners.get(case) != feature:
            errors.append(f"accounting case {case} does not name feature {feature}")
    for pattern, declaration in config.get("source_cases", {}).items():
        re.compile(pattern)
        if case_owners.get(declaration["case"]) != declaration["feature"]:
            errors.append(f"invalid component case {declaration}")
    for pattern, feature in config.get("source_features", {}).items():
        re.compile(pattern)
        if feature != "@entity" and feature not in names:
            errors.append(f"unknown source feature {feature}")
    return links, config, errors


@lru_cache
def rust_lexer(root: Path):
    # Reuse the repository's Rust lexer; comments, raw strings and nested block
    # comments cannot change the extent of a cfg(test) item.
    tool = root / "tools/check-semantic-contracts.py"
    if not tool.is_file():
        tool = Path(__file__).with_name("check-semantic-contracts.py")
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


def is_product_source(path: Path, internal: dict) -> bool:
    return (path.as_posix() not in internal and path.parts[0] == "src" and path.suffix == ".rs"
            and not any(part in {"test", "tests", "test_support", "generated"}
                        or re.search(r"(^tests?_|_tests?(?:\.rs)?$|^tests?\.rs$)", part)
                        for part in path.parts))


def coverage_report(root: Path, lcov: Path, statuses: dict, floor: float) -> dict:
    incomplete = {name: statuses.get(name, "missing") for name in ("rust", "python", "spec")
                  if statuses.get(name) != 0}
    if incomplete:
        return {"status": "incomplete", "collections": incomplete}
    _, config, errors = inventory(root)
    if errors:
        raise ValueError("\n".join(errors))
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
            if not is_product_source(candidate, config["internal"]):
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
    result = {"status": "pass" if percentage >= floor else "fail", "floor": floor,
            "denominator": "instrumented src/**/*.rs product lines; excludes test files, inline cfg(test) items, test_support and generated code",
            "product_lines": total, "reached_lines": reached,
            "outside_in_percent": percentage,
            "unobserved_files": sorted(str(p.relative_to(root)) for p in (root / "src").rglob("*.rs")
                                       if str(p.relative_to(root)) not in product
                                       and is_product_source(p.relative_to(root), config["internal"])),
            "unreached": {path: [number for number, hits in lines.items() if hits == 0]
                          for path, lines in product.items() if any(h == 0 for h in lines.values())}}

    result["excluded_test_sources"] = {path: reason for path, reason in config["internal"].items()
                                       if path.startswith("src/")}
    result["item_accounting"] = account_items(root, config, result, lcov.parent)
    return result


def account_items(root: Path, config: dict, result: dict, evidence: Path) -> dict:
    """Associate missing source observations with real cases, without claiming reach."""
    aliases = {"adverse_event": "adverse-event", "cell_line": "cell-line",
               "pharmacodb": "cell-line", "discover": "discovery",
               "section_outcome": "source-policy", "source_state_registry": "source-policy"}
    accounting = {}
    case_status = execution_status(evidence)
    for path in sorted(set(result["unreached"]) | set(result["unobserved_files"])):
        feature = None
        for pattern, owner in config.get("source_features", {}).items():
            match = re.match(pattern, path)
            if match:
                feature = aliases.get(match[1], match[1]) if owner == "@entity" else owner
                break
        if feature is None or feature not in config.get("accounting_cases", {}):
            raise ValueError(f"missing production feature/case association: {path}")
        observation = "unreached instrumented lines"
        if path in result["unobserved_files"]:
            text = (root / path).read_text()
            excluded = inline_tests(root, root / path)
            masked = rust_lexer(root)(text)
            executable = any(re.search(r"\bfn\s+\w+", line) for number, line in enumerate(masked.splitlines(), 1)
                             if number not in excluded)
            observation = ("runtime source absent from executable mapping" if executable else
                           "declarations/module glue absent from executable mapping")
        case = config["accounting_cases"][feature]
        for pattern, declaration in config.get("source_cases", {}).items():
            if re.match(pattern, path):
                feature, case = declaration["feature"], declaration["case"]
                break
        accounting[path] = {"feature": feature, "case": case,
                            "case_execution": case_status(case),
                            "observation": observation,
                            "remaining_limit": "The associated contract names the behavior; this collection does not establish that it reaches the missing lines."}
    return accounting


def execution_status(evidence: Path):
    """Read actual terminal suite evidence, without inferring execution from a link."""
    python = {}
    xml = evidence / "python.xml"
    if xml.is_file():
        for case in ET.parse(xml).iter("testcase"):
            key = case.attrib["classname"].replace(".", "/") + ".py::" + case.attrib["name"]
            python[key] = "skipped" if case.find("skipped") is not None else "passed"
    rust_log = evidence / "rust.log"
    rust = rust_log.read_text() if rust_log.is_file() else ""
    spec_log = evidence / "spec.log"
    spec = spec_log.read_text() if spec_log.is_file() else ""
    selection = re.search(r"^=== spec pages: (.*?) ===$", spec, re.M)
    selected_pages = selection[1].split() if selection else []

    def status(case: str) -> str:
        path, _, name = case.partition("::")
        if path.endswith(".py"):
            key = path + "::" + name
            matches = [value for test, value in python.items()
                       if test == key or ("[" not in key and test.startswith(key + "["))]
            if matches:
                return "Python suite: " + ", ".join(sorted(set(matches)))
            return "Python suite: no execution recorded" + ("; opt-in live module excluded" if "/live/" in path else "")
        if path.endswith(".rs"):
            found = re.search(r"^\s*(PASS|SKIP)\s+.*?::" + re.escape(Path(path).stem) + r"\s+" + re.escape(name) + r"$", rust, re.M)
            return "Rust suite: " + (found[1].lower() if found else "no execution recorded")
        return ("Spec suite: selected page passed" if path in selected_pages else
                "Spec suite: page excluded from offline selection; case unverified")
    return status
