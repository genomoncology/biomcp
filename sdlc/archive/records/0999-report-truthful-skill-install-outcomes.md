---
flow: quickfix
priority: 9
---

# Report truthful skill install outcomes

Skill installation must return a typed result whose status and `changed` value match the filesystem outcome. New installation, unchanged existing content, forced repair, and user cancellation are distinct outcomes. JSON includes the target and resulting skill status instead of hiding structured state inside human text.

Red-green coverage belongs in `src/cli/skill/tests/install.rs`, `src/cli/tests/facade.rs`, and `spec/surface/skills.md`; existing install output assertions may be restated.
