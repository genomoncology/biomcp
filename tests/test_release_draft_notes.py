from __future__ import annotations

import os
import subprocess
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"


def _steps(job: str) -> list[dict]:
    parsed = yaml.safe_load(RELEASE_WORKFLOW.read_text(encoding="utf-8"))
    return parsed["jobs"][job]["steps"]


def _step(steps: list[dict], needle: str) -> dict:
    for step in steps:
        if needle in (step.get("name") or step.get("uses") or ""):
            return step
    raise AssertionError(f"no step matching {needle!r}")


def test_the_draft_release_is_created_with_the_changelog_section_as_notes() -> None:
    """Ticket 2064: v0.9.2 and v0.9.1 published with no release notes.

    create-draft checks out the tag, renders the tag's changelog
    section, and creates the draft release with those notes, so a
    rehearsal reads the notes off the draft before it goes public.
    """
    steps = _steps("create-draft")
    checkout = _step(steps, "actions/checkout@")
    assert checkout["with"]["ref"] == "${{ inputs.tag || github.ref_name }}"
    render = _step(steps, "Render the release notes")
    assert "release/release_notes.py" in render["run"]
    assert "--changelog CHANGELOG.md" in render["run"]
    assert '--version "${TAG#v}"' in render["run"]
    create = _step(steps, "Create the draft release")
    assert 'gh release create "$TAG" --draft --verify-tag' in create["run"]
    assert '--notes-file "$RUNNER_TEMP/release-notes.md"' in create["run"]
    # publish-release only flips the draft public; it must not strip
    # the notes the draft already carries.
    publish = _step(_steps("publish-release"), "Publish the GitHub release")
    assert 'gh release edit "$TAG" --draft=false' in publish["run"]
    assert "--notes" not in publish["run"]


def test_the_notes_step_renders_the_exact_changelog_section(tmp_path: Path) -> None:
    """The render step's own script, executed against a checked-out
    release_notes.py and a synthetic tagged changelog, writes exactly
    the tag's curated section to the notes file."""
    render = _step(_steps("create-draft"), "Render the release notes")
    script = render["run"]
    assert "${{" not in script, "the render step must stay directly executable"
    checkout = tmp_path / "checkout"
    (checkout / "release").mkdir(parents=True)
    (checkout / "release" / "release_notes.py").write_text(
        (ROOT / "release" / "release_notes.py").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    (checkout / "CHANGELOG.md").write_text(
        "# Changelog\n\n"
        "## 0.9.3 — 2026-10-20\n\n### Fixes\n\n"
        "- Every documented install path works as the docs say. (2064)\n\n"
        "## 0.9.2 — 2026-10-11\n\n- Older release bullets.\n",
        encoding="utf-8",
    )
    runner_temp = tmp_path / "runner-temp"
    runner_temp.mkdir()
    result = subprocess.run(
        ["bash", "-e", "-c", script],
        cwd=checkout,
        capture_output=True,
        text=True,
        env=os.environ | {"TAG": "v0.9.3", "RUNNER_TEMP": str(runner_temp)},
        check=False,
    )
    assert result.returncode == 0, result.stderr
    notes = (runner_temp / "release-notes.md").read_text(encoding="utf-8")
    assert notes == (
        "## 0.9.3 — 2026-10-20\n\n### Fixes\n\n"
        "- Every documented install path works as the docs say. (2064)\n"
    )
