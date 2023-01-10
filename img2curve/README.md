# img2curve
A small tool written for [easyraw][er] used to create tone curves from images.

[er]: https://github.com/gennyble/easyraw

## Installation
Clone this repo and run `cargo install --path .`

## Usage
`img2curve <image>`

It wants an image of a curve as the first argument. It will then barf values from 0 to 1, inclusive, onto stdout. These values will be line separated.

### Making curves
![](curve.png)

For every pixel wide the image is, you'll get a value on the output. This value is the bottom-most black pixel for any X left of center and the top-most black pixel for any X right of center. These values are normalised against the height so they fall between, and including both, 0 and 1.

##### License
It's CC0 1.0, you can do whatever you want and and don't even need to give attribution; it's in the public domain.