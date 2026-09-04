import os
from pathlib import Path
import re
import shutil


REVISION = re.compile(r"^[0-9a-f]{40}$")
REVISION_DIRECTORY = "__biomcp_revision__"


def on_post_build(config, **kwargs) -> None:
    docs_dir = Path(config.docs_dir)
    site_dir = Path(config.site_dir)

    for source in docs_dir.rglob("*.md"):
        twin = site_dir / source.relative_to(docs_dir)
        twin.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, twin)

    witness_dir = site_dir / REVISION_DIRECTORY
    shutil.rmtree(witness_dir, ignore_errors=True)
    revision = os.environ.get("BIOMCP_DOCS_REVISION")
    if revision is None:
        return
    if REVISION.fullmatch(revision) is None:
        raise ValueError(
            "BIOMCP_DOCS_REVISION must be a 40-character lowercase hexadecimal SHA"
        )
    witness_dir.mkdir()
    (witness_dir / f"{revision}.txt").write_text(f"{revision}\n", encoding="utf-8")
