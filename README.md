**currently going through a major refactor everything is quite severely broken, but we're getting more colour accurate so that's cool :D**

# gaze
An image processing program that lives in the terminal.

## Structure
This repository contains gaze itself and a few other crates and tools that are tightly linked to it. The repository root is `gaze` itself; `/src` is gaze.

### `rawproc` ([readme](rawproc/README.md))
The crate containing the algorithms et al. for gaze.

### `img2curve` ([readme](img2curve/README.md))
We don't yet have a GUI for applying a tone curve, so this is a little tool that converts an image into a line-separated file for gaze to read.