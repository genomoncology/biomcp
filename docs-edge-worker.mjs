function acceptsMarkdown(header) {
  return header.split(",").some((entry) => {
    const [mediaType, ...parameters] = entry.split(";");
    const quality = parameters
      .map((parameter) => parameter.trim().match(/^q\s*=\s*(.+)$/i))
      .find(Boolean);
    return mediaType.trim().toLowerCase() === "text/markdown"
      && (!quality || Number.parseFloat(quality[1]) > 0);
  });
}

function addVaryAccept(headers) {
  const values = (headers.get("Vary") || "")
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  if (!values.some((value) => value.toLowerCase() === "accept")) {
    values.push("Accept");
  }
  headers.set("Vary", values.join(", "));
}

function markdownPaths(pathname) {
  if (pathname === "/") return ["/index.md"];
  if (pathname.endsWith("/")) {
    return [`${pathname.slice(0, -1)}.md`, `${pathname}index.md`];
  }
  return [];
}

async function assetResponse(request, env, pathname) {
  const url = new URL(request.url);
  url.pathname = pathname;
  return env.ASSETS.fetch(new Request(url, request));
}

async function findMarkdown(request, env, paths) {
  let result;
  for (const path of paths) {
    const response = await assetResponse(request, env, path);
    result = { path, response };
    if (response.ok) return result;
  }
  return result;
}

function withHeaders(response, update) {
  const headers = new Headers(response.headers);
  update(headers);
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname.endsWith(".md")) {
      const response = await env.ASSETS.fetch(request);
      if (!response.ok) return response;
      return withHeaders(response, (headers) => {
        headers.set("Content-Type", "text/markdown; charset=utf-8");
      });
    }

    const twinPaths = markdownPaths(url.pathname);
    if (twinPaths.length === 0) return env.ASSETS.fetch(request);

    if (acceptsMarkdown(request.headers.get("Accept") || "")) {
      const twin = await findMarkdown(request, env, twinPaths);
      if (!twin.response.ok) return twin.response;
      return withHeaders(twin.response, (headers) => {
        headers.set("Content-Type", "text/markdown; charset=utf-8");
        addVaryAccept(headers);
      });
    }

    const response = await env.ASSETS.fetch(request);
    const contentType = response.headers.get("Content-Type") || "";
    if (!response.ok || !contentType.toLowerCase().startsWith("text/html")) {
      return response;
    }
    const twin = await findMarkdown(request, env, twinPaths);
    if (!twin.response.ok) return response;
    return withHeaders(response, (headers) => {
      headers.set("X-Markdown-URL", new URL(twin.path, url.origin).href);
      addVaryAccept(headers);
    });
  },
};
