ARG RUNTIME_IMAGE=debian:trixie-slim@sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a
FROM ${RUNTIME_IMAGE}

ARG TARGETARCH
ARG SOURCE_SHA
ARG VERSION
ARG CREATED

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 65532 --create-home --home-dir /home/biomcp --shell /usr/sbin/nologin biomcp \
    && install -d -m 0700 -o biomcp -g biomcp \
       /home/biomcp/.cache/biomcp /home/biomcp/.config/biomcp

COPY --chmod=0755 dist/container/${TARGETARCH}/biomcp /usr/local/bin/biomcp
COPY dist/container/${TARGETARCH}/biomcp.sha256 /tmp/biomcp.sha256
RUN cd /usr/local/bin && sha256sum -c /tmp/biomcp.sha256 && rm /tmp/biomcp.sha256

LABEL org.opencontainers.image.source="https://github.com/genomoncology/biomcp" \
      org.opencontainers.image.revision="${SOURCE_SHA}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.created="${CREATED}"

USER 65532:65532
ENV HOME=/home/biomcp \
    XDG_CACHE_HOME=/home/biomcp/.cache \
    XDG_CONFIG_HOME=/home/biomcp/.config
ENTRYPOINT ["biomcp"]
