import csv
import hashlib
import json
import os
import ssl
from io import StringIO
from ssl import PROTOCOL_TLS_CLIENT, SSLContext, TLSVersion
from typing import Literal, TypeVar

import certifi
import httpx
from diskcache import Cache
from platformdirs import user_cache_dir
from pydantic import BaseModel

from . import const

T = TypeVar("T", bound=BaseModel)


class RequestError(BaseModel):
    code: int
    message: str


_cache: Cache | None = None


def get_cache() -> Cache:
    global _cache
    if _cache is None:
        cache_path = os.path.join(user_cache_dir("biomcp"), "http_cache")
        _cache = Cache(cache_path)
    return _cache


# noinspection PyTypeChecker
def generate_cache_key(method: str, url: str, params: dict) -> str:
    sha256_hash = hashlib.sha256()
    params_dump: str = json.dumps(params, sort_keys=True)
    key_source: str = f"{method.upper()}:{url}:{params_dump}"
    data: bytes = key_source.encode("utf-8")
    sha256_hash.update(data)
    return sha256_hash.hexdigest()


def cache_response(cache_key: str, content: str, ttl: int):
    expire = None if ttl == -1 else ttl
    cache = get_cache()
    cache.set(cache_key, content, expire=expire)


def get_cached_response(cache_key: str) -> str | None:
    cache = get_cache()
    return cache.get(cache_key)


def get_ssl_context(tls_version: TLSVersion) -> SSLContext:
    """Create an SSLContext with the specified TLS version."""
    context = SSLContext(PROTOCOL_TLS_CLIENT)
    context.minimum_version = tls_version
    context.maximum_version = tls_version
    context.load_verify_locations(cafile=certifi.where())
    return context


async def call_http(
    method: str,
    url: str,
    params: dict,
    verify: ssl.SSLContext | str | bool = True,
) -> tuple[int, str]:
    try:
        async with httpx.AsyncClient(verify=verify, http2=False) as client:
            if method.upper() == "GET":
                resp = await client.get(url, params=params)
            elif method.upper() == "POST":
                resp = await client.post(url, json=params)
            else:
                return 405, f"Unsupported method {method}"

        return resp.status_code, resp.text

    except httpx.HTTPError as exc:
        error_msg = str(exc) if str(exc) else "Network connectivity error"
        return 599, error_msg


async def request_api(
    url: str,
    request: BaseModel | dict,
    response_model_type: type[T] | None = None,
    method: Literal["GET", "POST"] = "GET",
    cache_ttl: int = const.DEFAULT_CACHE_TIMEOUT,
    tls_version: TLSVersion | None = None,
) -> tuple[T | None, RequestError | None]:
    verify = get_ssl_context(tls_version) if tls_version else True

    # Convert request to params dic
    if isinstance(request, BaseModel):
        params = request.model_dump(exclude_none=True, by_alias=True)
    else:
        params = request

    # Short-circuit if caching disabled
    if cache_ttl == 0:
        status, content = await call_http(method, url, params, verify=verify)
        return parse_response(status, content, response_model_type)

    # Else caching enabled:
    cache_key = generate_cache_key(method, url, params)
    cached_content = get_cached_response(cache_key)

    if cached_content:
        return parse_response(200, cached_content, response_model_type)

    # Make HTTP request if not cached
    status, content = await call_http(method, url, params, verify=verify)
    parsed_response = parse_response(status, content, response_model_type)

    # Cache if successful response
    if status == 200:
        cache_response(cache_key, content, cache_ttl)

    return parsed_response


def parse_response(
    status_code: int,
    content: str,
    response_model_type: type[T] | None = None,
) -> tuple[T | None, RequestError | None]:
    if status_code != 200:
        return None, RequestError(code=status_code, message=content)

    try:
        if response_model_type is None:
            if content.startswith("{") or content.startswith("["):
                response_dict = json.loads(content)
            elif "," in content:
                io = StringIO(content)
                response_dict = list(csv.DictReader(io))
            else:
                response_dict = {"text": content}
            return response_dict, None

        parsed: T = response_model_type.model_validate_json(content)
        return parsed, None

    except Exception as exc:
        return None, RequestError(
            code=500,
            message=f"Failed to parse response: {exc}",
        )
