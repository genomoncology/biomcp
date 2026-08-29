from pathlib import Path
import shutil


def on_post_build(config, **kwargs) -> None:
    docs_dir = Path(config.docs_dir)
    site_dir = Path(config.site_dir)

    for source in docs_dir.rglob("*.md"):
        twin = site_dir / source.relative_to(docs_dir)
        twin.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, twin)
