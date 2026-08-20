---
flow: build
priority: 15
---
# Stop flag order from silently changing a command

`get article 42125600 assets --asset-view coverage` fails with `Invalid argument: assets is a standalone JSON-only article section; do not combine it with other sections`. Moving the same flag before the section, `get article 42125600 --asset-view coverage assets`, works. The two forms differ only in argument order, and nothing tells the caller that order matters here.

The error compounds it by describing a problem the caller does not have. Nothing was combined with another section; the flag's value was parsed as one. A caller reading that message looks for a second section they never asked for.

The error object also comes back carrying `"assets": []` alongside the error. That is the shape a careless caller reads as "this article has no assets," which is the same class of mistake as reporting zero for a source that was never reached.

## Done when

- Both argument orders either work identically, or the rejected one fails with a message naming the actual cause.
- No error message for this command describes combining sections unless sections were actually combined.
- A response carrying an error does not also carry an empty collection under the key a successful call would populate, for this command's error paths.
