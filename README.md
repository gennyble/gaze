**currently going through a major refactor everything is quite severely broken, but we're getting more colour accurate so that's cool :D**

# gaze
An image processing program that lives in the terminal.

This repository contains gaze and associated libraries and programs.

### `gaze`
gaze itself. this directory might end up being replaced with `rawproc-dev`, but we'll see. the future is vast and uncertain.

### `curver` ([readme](curver/README.md))
A little GUI for creating tone curves. Saves as a line separated value. The readme has some more information and controls of the program.

---

### `rawproc` ([readme](rawproc/README.md))
The crate containing the algorithms et al. for gaze.

### `imgout`
Responsible for writing PNG, JPEG, and WebP files. Maybe more, later.

### `fluffy`
Image drawing thing for use with softbuffer.

---

### `dslr-trichrome` ([readme](dslr-trichrome/README.md))
Experiments in weird trichrome using my DSLR.

### `img2curve` ([readme](img2curve/README.md))
Little tool that reads an image and outputs a line-separated file for gaze to read a tone curve from.
