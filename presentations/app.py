#!/usr/bin/env python3
"""Flask slide deck server — multi-deck architecture.

Usage:
    python3 presentations/app.py       # Serve all decks on port 8881
    shot-scraper shot http://localhost:8881/intro/01-entities.html \
        -o out.png --width 1280 --height 720
"""
import os
from flask import Flask, render_template, jsonify, send_from_directory, abort
from jinja2 import FileSystemLoader
from decks import DeckRegistry, DECKS_DIR

app = Flask(
    __name__,
    static_folder=os.path.join(DECKS_DIR, "static"),
    template_folder=os.path.join(DECKS_DIR, "templates"),
)

registry = DeckRegistry()

app.jinja_loader = FileSystemLoader([
    os.path.join(DECKS_DIR, "templates"),
    os.path.join(DECKS_DIR, "content"),
])


@app.route("/")
def home():
    return render_template("index.html", decks=registry.all())


@app.route("/<deck_id>/")
def deck_index(deck_id):
    deck = registry.get(deck_id)
    if not deck:
        abort(404)
    return render_template(
        "index.html",
        decks=registry.all(),
        sections=deck.get_sections(),
        deck=deck,
    )


@app.route("/<deck_id>/<slug>.html")
def slide(deck_id, slug):
    deck = registry.get(deck_id)
    if not deck:
        abort(404)
    meta = deck.slide_map.get(slug)
    if not meta:
        abort(404)
    return render_template(
        f"{deck_id}/slides/{slug}.html", slide=meta, slides=deck.slides, deck=deck,
    )


@app.route("/<deck_id>/nav.js")
def nav_js(deck_id):
    deck = registry.get(deck_id)
    if not deck:
        abort(404)
    return (
        render_template("nav.js", slides=deck.slides, deck_id=deck_id),
        200,
        {"Content-Type": "application/javascript"},
    )


@app.route("/<deck_id>/assets/<path:filename>")
def deck_assets(deck_id, filename):
    deck = registry.get(deck_id)
    if not deck:
        abort(404)
    assets_dir = os.path.join(deck.content_dir, "assets")
    return send_from_directory(assets_dir, filename)


@app.route("/themes/<path:filename>")
def themes(filename):
    return send_from_directory(os.path.join(DECKS_DIR, "themes"), filename)


@app.route("/api/decks")
def api_decks():
    return jsonify([
        {"id": d.id, "title": d.title, "slides": len(d.slides),
         "status": d.config.get("status", "draft")}
        for d in registry.all()
    ])


@app.route("/api/decks/<deck_id>/slides")
def api_slides(deck_id):
    deck = registry.get(deck_id)
    if not deck:
        abort(404)
    return jsonify([
        {"slug": s["slug"], "title": s["title"], "section": s["section"]}
        for s in deck.slides
    ])


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8881, debug=True)
