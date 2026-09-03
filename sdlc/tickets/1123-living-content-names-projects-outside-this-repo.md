---
flow: build
priority: 2
---

# Living content names projects outside this repo

Four lines in living, non-ticket content name a private project or a path in another workspace repository. A reader outside the ecosystem cannot resolve either, and the repository is meant to read self-contained.

The four places, verified on 2026-09-02 and re-verified on 2026-09-03. Each is named by file and by what the sentence says, not by line number, because a line number rots the moment anything above it moves:

- `architecture/experiments/structural-variant-article-annotations/explore.md` names two private projects in its schema-alignment bullet, in the phrase describing an alteration grammar.
- `architecture/experiments/structural-variant-article-annotations/harden.md` names the same two projects in the sentence listing consumers, and in the same sentence cites a spike plan by its path in another workspace repository.
- `sdlc/issues/2026-08-26-drug-mechanism-shows-pharmacologic-action.md` cites a capture file by its path in another workspace repository.
- `sdlc/issues/2026-08-26-search-all-pathways-ignores-query.md` cites the same capture file the same way.

## Required behavior

Living content in this repository names no project outside it and cites no path outside it.

Replace each private project name with a neutral description of the consumer. "A downstream alteration-grammar consumer" carries the meaning without the name.

For the two capture citations, either copy the capture into this repository and cite the local copy, or restate what the capture showed and drop the path. Do not leave a path a reader cannot open.

## Done, observably

- None of the four lines names a private project or a path outside this repository.
- The meaning of each sentence survives. A reader still learns who the consumer is and what the capture showed.
- No other file in living content acquires such a name.

## Boundary

Change only the four files listed above. Find each occurrence by searching the file for the private name; do not rely on a line number.

Do not touch `sdlc/records/` or `sdlc/tickets/archive/`. History is append-only and stays exactly as written.

Do not touch any file under `sdlc/tickets/`. An active ticket also carries such a name, and the landing gate forbids changing it. That line leaves when its ticket is archived. Ticket 1081 was withdrawn on 2026-09-02 because it bundled that unreachable change with these four; this ticket is the reachable part.

The `genomoncology` organization name stays wherever it is this repository's own public hosting, packaging, or attribution.
