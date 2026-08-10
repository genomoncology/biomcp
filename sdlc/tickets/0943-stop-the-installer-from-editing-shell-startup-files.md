---
flow: quickfix
priority: 6
deps: ["0911", "0938"]
---
# Stop the installer from editing shell startup files

The standalone installer silently appends PATH setup to shell startup files,
while installation documentation says only that it places the binary. A
download-and-install command should not make unrelated persistent shell
configuration changes without an explicit request.

## Installer contract

The canonical installer never creates or edits `.bashrc`, `.bash_profile`,
`.zshrc`, `.profile`, or another startup file. When the destination is absent
from PATH, print one shell-appropriate command the user may copy, together with
the installed path. When it is already reachable, print no PATH warning.

Do not add an automatic opt-in flag in this ticket: printing instructions is
the complete simple behavior and avoids symlink, ownership, shell-detection,
and marker-block machinery.

## Done when

- Installer tests use sentinel startup files for supported shells and prove
  new install, upgrade, success, and failure leave every byte and mode intact.
- PATH-present and PATH-absent cases print the correct concise result without
  sourcing a file or changing the current process environment.
- Documentation describes binary placement and the manual PATH step exactly.
- The deployed installer identity from 0911 and atomic transaction from 0938
  remain intact.

## Authorized test changes

The quickfix may remove startup-file mutation from canonical `install.sh` and
restate installer/output/documentation tests that currently preserve it. Shell
startup files must not become part of the installer ownership receipt. No
product Rust source change belongs here.

The src line ceiling may not rise.
