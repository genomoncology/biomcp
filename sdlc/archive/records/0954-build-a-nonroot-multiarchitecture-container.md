---
base: 777a72fe8956d1fdbb66fb881a98686c645f80ad
head: e96cb5f98b69f3d4cd3a1d60b07a99d967a230ee
---

Replaced source compilation inside the Docker build with a runtime-only image
assembled from the two registered Linux executables. The build context contains
only the Dockerfile and staged container inputs.

The pinned image has private writable state, a non-root UID, no exposed port,
the BioMCP entrypoint, exact revision/version labels, two-platform OCI
inspection, private platform smokes, and no stage-time registry push.
