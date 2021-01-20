use rawproc::image::{Component, Image, Metadata, Sensor};

pub fn subsample<T: Component>(rimg: Image<Sensor, T>) -> Image<Sensor, T> {
    // Assuming RGGB and a 1/4 scale
    let mut raw = vec![];

    // These are dimensions in CFA groups
    let width = rimg.meta.width / 8;
    let height = rimg.meta.height / 8;

    for y in 0..height {
        let j = y as usize * 8 * rimg.meta.width as usize;
        for x in 0..width {
            let i = j + (x as usize * 8);

            raw.push(rimg.data[i]);
            raw.push(rimg.data[i + 1]);
        }

        for x in 0..width {
            let i = j + (x as usize * 8) + rimg.meta.width as usize;

            raw.push(rimg.data[i]);
            raw.push(rimg.data[i + 1]);
        }
    }

    let width = width * 2;
    let height = height * 2;

    assert_eq!(width as usize * height as usize, raw.len());

    Image {
        kind: Sensor {},
        data: raw,
        meta: Metadata::new(width, height, rimg.meta.cfa, rimg.meta.colordata),
    }
}
