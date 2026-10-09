# The release line flows into 1.0, never the reverse

October 9, 2026. Ruled by Ian.

## The ruling

Ian, overruling a proposal to land 1.0 onto the release line and cut a maintenance branch from it:

> `main` is what people will download and start working from if they want to use BioMCP and they want to use the latest code. The latest code is 0.9.2, and then it will be 0.9.3, so it's really imperative that 1.0 pulls `main` into its branch continuously, not the other way around.

So `main` is the public download surface and stays that way. The 1.0 line is a long-lived branch that merges `main` in, at least daily, and keeps its drift near zero.

## What this replaces

A proposal to merge the 1.0 branch into `main` and cut a maintenance branch for the 0.9 line. That proposal treated `main` as a development trunk. It is not; it is what a stranger installs.

## What follows from it

The 1.0 branch never pushes code to `main`.

Drift is a defect, not a state. A branch hundreds of commits behind the release line is carrying fixes its users will never get and conflicts nobody has resolved.

1.0 tickets live on `main`, in `main`'s ticket folder, stamped with the 1.0 milestone, so one ticket database serves both lines and numbers cannot collide. Only records go to `main` from the 1.0 lane. Code does not.

The 0.9 release line is driven by a separate lane with its own order. This repository's plan covers the 1.0 destination; it does not set that lane's priorities.
