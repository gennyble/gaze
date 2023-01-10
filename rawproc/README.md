# rawproc

I wanted to try and process the raw NEF files that my camera gave me using my own code, so I wrote
some. Then I put it into a library and now you're reading the readme for that library.

The interpolation algorithm in use currently assumes an RGGB color filter array. This'll might
work for your camera, but it also might not.

The docs are kind of lacking (but will be improved!), so if you've somehow found this repository
and want to use this crate, the code over in the [easyraw repository][easyraw-github] is a pretty
good example.

[easyraw-github]: https://github.com/gennyble/easyraw

## Testing
The tests and benchmarks require raw files that I don't want to stuff into this repository because
of their size. They are located here: <https://nyble.dev/rawproc/testfiles.zip>. Extract that to
`tests`. It should look like `tests/raw/<lots of raw images>`.

## Operations

Going from a `SensorImage<f32>` to an `RgbImage<f32>`.
```rust
use rawproc::{debayer::{Debayer, Interpolation};

Debayer::new(sensor_floats).interpolate(Interpolation::Bilinear);
```

| Image Op.       | Rgb     | Hsv     | Gray    | Sensor       |
| --------------- | ------- | ------- | ------- | ------------ |
| Black Level     | x       | x       | x       | u8, u16, f32 |
| White Balance   | x       | x       | x       | f32          |
| Exposure        | x       | x       | x       | f32          |
| Simple Gamma    | x       | x       | x       | f32          |
| Hue Shift       | x       | f32     | x       | x            |
| Saturation      | x       | f32     | x       | x            |
| Brightness      | x       | f32     | x       | x            |
| sRGB Conversion | f32     | x       | x       | x            |
| Contrast        | f32     | x       | x       | x            |