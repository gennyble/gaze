**currently going through a major refactor everything is quite severely broken, but we're getting more colour accurate so that's cool :D**

# gaze
An image processing program that lives in the terminal.

## Structure
This repository contains gaze itself and a few other crates and tools that are tightly linked to it. The repository root is `gaze` itself; `/src` is gaze.

### `curver` ([readme](curver/README.md))
A little GUI for creating tone curves. Saves as a line separated value. The readme has some more information and controls of the program.

### `dslr-trichrome` ([readme](dslr-trichrome/README.md))
Experiments in weird trichrome using my DSLR.

### `rawproc` ([readme](rawproc/README.md))
The crate containing the algorithms et al. for gaze.

### `img2curve` ([readme](img2curve/README.md))
Little tool that reads an image and outputs a line-separated file for gaze to read a tone curve from.

### `imgout`
Responsible for writing PNG, JPEG, and WebP files. Maybe more, later.
