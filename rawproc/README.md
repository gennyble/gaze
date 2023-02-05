# rawproc

I wanted to try and process the raw NEF files that my camera gave me using my own code, so I wrote
some. Then I put it into a library and now you're reading the readme for that library.

I want to try and do color stuff myself, but if I get stuck I will probably look at palette ([github](https://github.com/Ogeon/palette))

- Everything on [brucelindbloom.com](http://www.brucelindbloom.com/). Bruce is a color scientist.
- 4.2.1 of [this pdf](https://faculty.kfupm.edu.sa/ics/lahouari/Teaching/colorspacetransform-1.0.pdf)
- ["Completely Painless Programmer's Guide to XYZ, RGB, ICC, xyY, and TRCs"](https://ninedegreesbelow.com/photography/xyz-rgb.html#xyY)
- [poynton Color FAQ](http://poynton.ca/notes/colour_and_gamma/ColorFAQ.html)
- <https://www.strollswithmydog.com/determining-forward-color-matrix/> and the rest of the stuff there.
- white balance "tint" <https://www.dpreview.com/forums/thread/4385471>

## Questions still unanswered
If you know this, think you figured it out, or otherwise just want to email me. Please. My head is exploding and I must know the answers. [gen@nyble.dev](mailto:gen@nyble.dev).

What is "unity" as referenced in poynton's Color FAQ [here](http://poynton.ca/notes/colour_and_gamma/ColorFAQ.html#RTFToC4)?

## Testing
The tests and benchmarks require raw files that I don't want to stuff into this repository because
of their size. They are located here: <https://nyble.dev/rawproc/testfiles.zip>. Extract that to
`tests`. It should look like `tests/raw/<lots of raw images>`.

## Operations
The three major types we recognize are u8, u16, and f32.

Whitebalance:
- BayerRgb: u8, u16, f32
- LinRgb
