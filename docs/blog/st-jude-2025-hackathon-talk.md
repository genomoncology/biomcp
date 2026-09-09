# The 2025 KIDS BioHackathon Winner Used BioMCP

*The talk that came out of it is finally on YouTube.*

**TL;DR:** A project that used BioMCP won St. Jude's KIDS BioHackathon in autumn 2025. The invitation to speak followed. That talk is now published, and one prediction in it turned into the shape BioMCP has today.

<div style="position:relative;padding-bottom:56.25%;height:0;overflow:hidden;max-width:100%;margin:1.5rem 0;">
  <iframe style="position:absolute;top:0;left:0;width:100%;height:100%;border:0;"
    src="https://www.youtube-nocookie.com/embed/lXoe-4TENDE"
    title="BioMCP: An Introduction to Biomedical AI Agents (October 2025)"
    allow="accelerometer; clipboard-write; encrypted-media; picture-in-picture"
    allowfullscreen></iframe>
</div>

[Watch on YouTube](https://www.youtube.com/watch?v=lXoe-4TENDE)

---

## How the talk happened

St. Jude Children's Research Hospital runs a KIDS BioHackathon. In 2025 a winning project used BioMCP as part of its technology stack. Their data science office invited Ian Maurer to speak to the community of practice a few weeks later, on October 24, 2025.

The talk is an introduction to BioMCP for researchers who had just watched a colleague use it to win something. It assumes no background in agents.

---

## What it covers

The first half is orientation. Pre-training, prompting, retrieval, post-training, and the reasoning models that had arrived that year. The argument underneath is simple. A model answering from memory guesses. A model that can retrieve evidence and call tools does better work, and in biomedicine the difference decides whether the answer is usable.

The second half is BioMCP against real sources. Clinical trial search that turns informal language and drug aliases into a structured ClinicalTrials.gov query. Literature retrieval through PubTator, which recognises genes, drugs, diseases, and variants as entities. Variant lookups across several genomic databases.

Two ideas from it have aged well:

**Curated knowledge and language models fix each other's weaknesses.** A knowledge graph gives provenance, structure, and an explanation you can trace. A language model gives flexible language and synthesis. Neither is sufficient. The talk argues for the hybrid, and BioMCP is built on that premise.

**Tool output is part of the security boundary.** An untrusted server, or untrusted content returned through a trusted one, can steer a model. There is a live demo of a model with web search enabled wandering off its intended sources mid-answer. Turning on a tool is a decision about what counts as evidence, not only a decision about capability.

---

## The prediction

At [54:46](https://www.youtube.com/watch?v=lXoe-4TENDE&t=3286s) the talk makes a call that felt risky at the time:

> That CLI might actually be the new MCP.

The reasoning was practical rather than visionary. Ian had built the command line interface mostly so he could write automated tests without driving MCP tools, then noticed agents used it more comfortably than the protocol.

A year later that is what BioMCP is. One Rust binary. One command grammar. The MCP server is a single tool that walks the same grammar the CLI does, so the model learns one thing instead of thirty-five. That story is told in [We Deleted 35 Tools and Our Agent Got Better](we-deleted-35-tools.md).

---

## What has changed since the recording

The talk is a year old and honest about its own moment. Some of it is now out of date:

- BioMCP is described as a Python library with roughly 24 tools. It is a Rust binary with one grammar and about 30 sources.
- Claude Skills had shipped eight days before the talk. Ian mentions them at [55:56](https://www.youtube.com/watch?v=lXoe-4TENDE&t=3356s) as something that would be fun to build for BioMCP. Skills now ship with the project.
- The model names, prices, and benchmark charts are a time capsule. The architectural conclusion under them still holds.

The parts that held up are the ones about interfaces, evidence boundaries, and validation. Better models did not remove the need for any of them.

---

## Chapters

| Time | Section |
| --- | --- |
| [00:00](https://www.youtube.com/watch?v=lXoe-4TENDE&t=0s) | Introduction: GenomOncology and precision oncology |
| [02:48](https://www.youtube.com/watch?v=lXoe-4TENDE&t=168s) | The pre-training era |
| [07:45](https://www.youtube.com/watch?v=lXoe-4TENDE&t=465s) | Prompting techniques and RAG |
| [13:00](https://www.youtube.com/watch?v=lXoe-4TENDE&t=780s) | Post-training and reasoning models |
| [19:46](https://www.youtube.com/watch?v=lXoe-4TENDE&t=1186s) | Knowledge graphs meet language models |
| [26:20](https://www.youtube.com/watch?v=lXoe-4TENDE&t=1580s) | Model Context Protocol |
| [31:41](https://www.youtube.com/watch?v=lXoe-4TENDE&t=1901s) | What BioMCP is |
| [35:46](https://www.youtube.com/watch?v=lXoe-4TENDE&t=2146s) | Demos: trials, literature, variants |
| [48:27](https://www.youtube.com/watch?v=lXoe-4TENDE&t=2907s) | The biomedical research assistant |
| [53:07](https://www.youtube.com/watch?v=lXoe-4TENDE&t=3187s) | Coding agents and what comes next |

A full transcript with chapter links sits on [Ian's site](https://www.imaurer.com/talks/biomcp-st-jude/).
