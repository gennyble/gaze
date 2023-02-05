# rawproc

I wanted to try and process the raw NEF files that my camera gave me using my own code, so I wrote
some. Then I put it into a library and now you're reading the readme for that library.

The interpolation algorithm in use currently assumes an RGGB color filter array. This'll might
work for your camera, but it also might not.

The docs are kind of lacking (but will be improved!), so if you've somehow found this repository
and want to use this crate, the code over in the [easyraw repository][easyraw-github] is a pretty
good example.

[easyraw-github]: https://github.com/gennyble/easyraw

I want to try and do color stuff myself, but if I get stuck I will probably look at palette ([github](https://github.com/Ogeon/palette))

- Everything on [brucelindbloom.com](http://www.brucelindbloom.com/). Bruce is a color scientist.
- 4.2.1 of [this pdf](https://faculty.kfupm.edu.sa/ics/lahouari/Teaching/colorspacetransform-1.0.pdf)
- ["Completely Painless Programmer's Guide to XYZ, RGB, ICC, xyY, and TRCs"](https://ninedegreesbelow.com/photography/xyz-rgb.html#xyY)

## Testing
The tests and benchmarks require raw files that I don't want to stuff into this repository because
of their size. They are located here: <https://nyble.dev/rawproc/testfiles.zip>. Extract that to
`tests`. It should look like `tests/raw/<lots of raw images>`.

## Operations
The three major types we recognize are u8, u16, and f32.

Whitebalance:
- BayerRgb: u8, u16, f32
- LinRgb