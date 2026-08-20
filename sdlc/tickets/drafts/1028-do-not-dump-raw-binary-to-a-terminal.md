---
flow: build
priority: 3
hold: draft for review; do not promote until Ian releases this
---
# Do not dump raw binary to a terminal

`get article <id> asset <key>` writes the retrieved bytes straight to standard output with no warning. Fetching a `.docx` supplement fills the terminal with ZIP container bytes, which at best is unreadable and at worst leaves the terminal in a broken state.

Writing bytes to standard output is the right behavior when the caller is piping them somewhere. The problem is doing it when the caller is a person looking at a terminal, and having no obvious way to ask for a file instead.

## The hard choice to settle

Decide between refusing to write binary to an interactive terminal unless the caller opts in, and adding an explicit destination flag, and doing both. Whichever is chosen must keep piping to a file or another process working exactly as it does now, because that is the case the command exists to serve.

## Done when

- Running the command interactively against a binary asset does not fill the terminal with raw bytes by default.
- Redirecting or piping the command's output still produces the exact bytes it produces today, unchanged.
- The help text says where the bytes go and how to change that.
- Whatever guard is added does not depend on the file extension, since the media type is already known from the manifest.
