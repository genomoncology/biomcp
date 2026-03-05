#!/usr/bin/env python3
"""Deck registry — loads YAML configs, builds slide metadata."""
import os
import yaml

DECKS_DIR = os.path.dirname(os.path.abspath(__file__))


class Deck:
    def __init__(self, deck_id: str, config: dict, content_dir: str):
        self.id = deck_id
        self.config = config
        self.title = config["title"]
        self.subtitle = config.get("subtitle", "")
        self.author = config.get("author", "")
        self.date = config.get("date", "")
        self.theme = config.get("theme", "go-brand")
        self.footer = config.get("footer", "")
        self.content_dir = content_dir

        self.sections = config.get("sections", [])
        self.slides = []
        for section in self.sections:
            for slide_def in section.get("slides", []):
                slide = dict(slide_def)
                slide["section"] = section["name"]
                self.slides.append(slide)

        for i, slide in enumerate(self.slides):
            slide["page_num"] = i + 1
            slide["total"] = len(self.slides)
            slide["prev"] = self.slides[i - 1]["slug"] if i > 0 else None
            slide["next"] = (
                self.slides[i + 1]["slug"]
                if i < len(self.slides) - 1
                else None
            )
            slide["deck_id"] = self.id
            slide["deck_footer"] = self.footer
            slide.setdefault("gradient", "gradient-ocean")

        self.slide_map = {s["slug"]: s for s in self.slides}

    def get_sections(self):
        sections = []
        current = None
        for s in self.slides:
            if s["section"] != current:
                current = s["section"]
                sections.append({"name": current, "slides": []})
            sections[-1]["slides"].append(s)
        return sections


class DeckRegistry:
    def __init__(self):
        self.decks: dict[str, Deck] = {}
        self._load_all()

    def _load_all(self):
        content_dir = os.path.join(DECKS_DIR, "content")
        if not os.path.isdir(content_dir):
            return
        for entry in sorted(os.listdir(content_dir)):
            deck_yaml = os.path.join(content_dir, entry, "deck.yaml")
            if os.path.isfile(deck_yaml):
                with open(deck_yaml) as f:
                    config = yaml.safe_load(f)
                self.decks[entry] = Deck(
                    entry, config, os.path.join(content_dir, entry)
                )

    def get(self, deck_id: str):
        return self.decks.get(deck_id)

    def all(self) -> list[Deck]:
        return list(self.decks.values())
