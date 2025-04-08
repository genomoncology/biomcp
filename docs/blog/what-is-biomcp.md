# What is BioMCP?

In the rapidly evolving landscape of artificial intelligence, the power of
large language models (LLMs) like ChatGPT and Claude has transformed how we
interact with information. However, these models face a significant limitation:
without context, they remain static and incomplete, especially in complex
domains like healthcare and biomedical research.

This is where BioMCP comes in – an open-source implementation of the Model
Context Protocol (MCP) specifically designed for biomedical applications. But
what exactly does that mean, and why does it matter?

## Bridging the Gap Between AI and Specialized Knowledge

BioMCP serves as a crucial bridge connecting AI assistants and agents to
specialized biomedical data sources. While LLMs have been trained on vast
amounts of public data and now have web search capabilities, they often lack
the specialized context needed for biomedical research.

The Model Context Protocol, recently released by Anthropic, has emerged as a
standard for empowering LLMs with tools, resources, and prompts. BioMCP builds
on this foundation, creating a specialized toolbox that enables AI systems to
access and interpret complex biomedical information through natural language
conversation.

## What Can BioMCP Access?

BioMCP provides seamless connections to three critical biomedical resources:

1. **Clinical Trials** - Through the ClinicalTrials.gov API, researchers can
   discover active trials for specific drugs, diseases, or locations. The AI
   assistant parses natural language queries into structured search parameters,
   retrieving and explaining relevant trials.

2. **Genomic Variants** - Using the MyVariant.info API, BioMCP enables rich
   filtering, sorting, and identification of specific genomic variants, making
   complex genomic data accessible through conversation.

3. **Research Literature** - BioMCP connects to PubMed through PubTator3, which
   excels at recognizing biomedical entities like drugs, diseases, genes, and
   variants, dramatically improving search relevance and retrieval.

## How Does It Transform Research?

What makes BioMCP particularly powerful is its conversational nature. A
researcher might begin with a simple question about a disease, then naturally
progress to exploring related clinical trials, and finally investigate genetic
variants that affect treatment efficacy—all within a single, flowing
conversation.

The system remembers context throughout the interaction, allowing for natural
follow-up questions and a research experience that mirrors how scientists
actually work. Instead of requiring researchers to master complex query
languages for each database, BioMCP translates natural language into the
precise syntax each system requires.

## Why This Matters

BioMCP represents a significant advancement in making specialized biomedical
knowledge accessible. For researchers and clinicians, it means spending less
time wrestling with complex database interfaces and more time advancing their
work. For the broader field of AI in healthcare, it demonstrates how
specialized knowledge domains can be made accessible through conversation.

As both AI assistants (synchronous conversation partners) and AI agents (
autonomous systems working toward goals over time) continue to evolve, tools
like BioMCP will be essential in connecting these systems to the specialized
knowledge they need to deliver meaningful insights in complex domains.

By open-sourcing BioMCP, we're inviting the community to build upon this
foundation, creating more powerful and accessible tools for biomedical research
and ultimately accelerating the pace of scientific discovery.
