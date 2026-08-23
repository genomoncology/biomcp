---
base: b125c4f680cf07fbe2d4674cf5ebe9d4cf9c3f4f
head: c1c91a18bcbb79e185479d12d374f553fd070028
---

# Do not dump raw binary to a terminal

Article assets now refuse binary or unknown media types on an interactive
terminal, while piping preserves their exact bytes. The article command
advertises `--output FILE` for atomic, user-selected file output.
