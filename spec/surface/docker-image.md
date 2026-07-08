# Docker Image

BioMCP ships as a container image for users who want the CLI or stdio MCP server
without installing a local Rust or Python toolchain. The image should behave like
the local binary: Docker supplies the executable, and users pass normal `biomcp`
arguments after the image name.

## Docker Image Uses The BioMCP Entrypoint

The root Dockerfile is the source of the published image. It should keep HTTPS
trust roots in the runtime layer and expose the `biomcp` binary directly as the
entrypoint so documented `docker run` commands can pass ordinary CLI arguments.

```bash
find ../.. -maxdepth 1 -name Dockerfile -type f -exec sed -n '1,220p' {} \; | mustmatch like 'ca-certificates
ENTRYPOINT ["biomcp"]'
```

## Docker Context Excludes Local Artifacts

The image build context should not send local build outputs, caches, March
runtime state, or the Git database to the Docker daemon. Keeping those paths out
prevents slow builds and avoids leaking local-only files into image layers.

```bash
find ../.. -maxdepth 1 -name .dockerignore -type f -exec sed -n '1,120p' {} \; | mustmatch like 'target/
.cache/
.march/
.git/'
```

## Pull Request CI Builds And Smokes The Image

Pull request CI should build the image and run the two no-network smoke commands
that prove the entrypoint works: version output and the command-reference page.
Keeping this in CI catches broken Dockerfiles before a release tries to publish
an unusable image.

```bash
sed -n '1,260p' ../../.github/workflows/ci.yml | mustmatch like 'docker build
docker run --rm
--version
list'
```

## Release Publishes GHCR Image Tags

The release workflow should publish the official image to GHCR with both the
release version tag and `latest`. Versioned tags give users a stable pin, while
`latest` keeps the simplest getting-started command current.

```bash
awk '/^  docker-publish:/{seen=1} seen && /^  [A-Za-z0-9_-]+:/{if ($1 != "docker-publish:") exit} seen {print}' ../../.github/workflows/release.yml | mustmatch like 'Sync Docker image version from release tag
Cargo.toml
ghcr.io/genomoncology/biomcp
type=semver
type=raw,value=latest
docker/build-push-action'
```

The workflow also grants package publishing permission at the top level so the
GHCR push can succeed when the release job runs.

```bash
sed -n '1,30p' ../../.github/workflows/release.yml | mustmatch like 'packages: write'
```

## Documentation Shows Docker CLI And Stdio MCP Use

The user docs should show both ways people run the image: direct CLI commands
such as `--version` and `list`, and a stdio MCP server invocation that passes
`serve` to the same image entrypoint.

```bash
cat ../../README.md ../../docs/getting-started/installation.md ../../docs/reference/mcp-server.md | mustmatch like 'docker run --rm ghcr.io/genomoncology/biomcp --version
docker run --rm ghcr.io/genomoncology/biomcp list
docker run --rm -i ghcr.io/genomoncology/biomcp serve'
```
