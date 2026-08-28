---
---
# Bound the unparsed gate verdict to 1,024 bytes

When `sdlc/project/before` cannot parse a failed gate as TAP, its fallback
verdict can occupy 1,025 bytes. The fallback reserves its payload budget from
the header length but does not reserve the newline after that header.

Keep the complete fallback verdict, including its header and final newline,
within the documented 1,024-byte limit. Add an executable lifecycle contract
for a long non-TAP gate failure so this edge case cannot regress.
