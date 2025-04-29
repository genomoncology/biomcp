"""Cloudflare Workers module for BioMCP."""

from .worker import create_worker_app

__all__ = ["create_worker_app"]
