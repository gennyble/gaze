# img2curve
A small tool written for gaze! Used to create tone curves from images.

## Installation
Clone this repo and run `cargo install --path img2curve`

## Usage
`img2curve <image>`

It wants an image of a curve as the first argument. It will then output values from 0 to 1, inclusive, onto stdout. These values will be line separated.

### Making curves
For every pixel wide the image is, you'll get a value on the output. This value is the bottom-most black pixel for any X left of center and the top-most black pixel for any X right of center. These values are normalized against the height so they fall between, and including both, 0 and 1.
