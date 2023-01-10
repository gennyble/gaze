#[inline(always)]
pub fn contrast(sample: &mut f32, adjustment: f32) {
    *sample = (adjustment * (*sample - 0.5) + 0.5).clamp(0.0, 1.0);
}

pub fn pixel_rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let value = r.max(g.max(b));
    let x_min = r.min(g.min(b));
    let chroma = value - x_min;

    let hue = if chroma == 0.0 {
        0.0
    } else if value == r {
        60.0 * ((g - b) / chroma)
    } else if value == g {
        60.0 * (2.0 + (b - r) / chroma)
    } else if value == b {
        60.0 * (4.0 + (r - g) / chroma)
    } else {
        unreachable!()
    };

    let value_saturation = if value == 0.0 { 0.0 } else { chroma / value };

    /* Rotate the color wheel counter clockwise to remove the negative location
          |       Keep the wheel in place and remove any full rotations
     _____V____ _____V____
    |          |          |*/
    ((hue + 360.0) % 360.0, value_saturation, value)
}

pub fn pixel_hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (f32, f32, f32) {
    let chroma = value * saturation;
    let hue_prime = hue / 60.0;
    let x = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());

    let m = value - chroma;
    let cm = chroma + m;
    let xm = x + m;

    if 0.0 <= hue_prime && hue_prime <= 1.0 {
        (cm, xm, m)
    } else if 1.0 < hue_prime && hue_prime <= 2.0 {
        (xm, cm, m)
    } else if 2.0 < hue_prime && hue_prime <= 3.0 {
        (m, cm, xm)
    } else if 3.0 < hue_prime && hue_prime <= 4.0 {
        (m, xm, cm)
    } else if 4.0 < hue_prime && hue_prime <= 5.0 {
        (xm, m, cm)
    } else if 5.0 < hue_prime && hue_prime <= 6.0 {
        (cm, m, xm)
    } else {
        panic!(
            "This shouldn't be able to happen! HSV ({},{},{})",
            hue, saturation, value
        );
    }
}

#[cfg(test)]
mod cfa_tets {
    use super::*;

    // Simple colors. Maxed Red, Green, and Blue
    #[test]
    fn rgb_to_hsv_simple() {
        // White. No saturation (no "colorfullness") and all value (all light emmitted, kind of)
        let (_h, s, v) = pixel_rgb_to_hsv(1.0, 1.0, 1.0);
        assert_eq!((s, v), (0.0, 1.0));

        // Full Red. Hue at 0 degrees, all colorfullness and all lit
        assert_eq!(pixel_rgb_to_hsv(1.0, 0.0, 0.0), (0.0, 1.0, 1.0));

        // Full Green
        assert_eq!(pixel_rgb_to_hsv(0.0, 1.0, 0.0), (120.0, 1.0, 1.0));

        // Full Blue
        assert_eq!(pixel_rgb_to_hsv(0.0, 0.0, 1.0), (240.0, 1.0, 1.0));
    }

    // More complex colors (not just 0 and 1)
    #[test]
    fn rgb_to_hsv() {
        fn assert_close(a: (f32, f32, f32), b: (f32, f32, f32)) {
            let tolerance = 3.9e-3; // 1 step in 8bit color
            if (a.0 - b.0).abs() > tolerance
                || (a.1 - b.1).abs() > tolerance
                || (a.2 - b.2).abs() > tolerance
            {
                panic!(
                    "assertion failed: `(left ~ right)`\n\
					\tLeft: `{:?}`,\n\
					\tRight:`{:?}`\n\
					Deviation allowed from left (tolerance is {}):\n\
					\tMax: `{:?}`\n\
					\tMin: `{:?}`\n\
					Actual Deviation: `{:?}`",
                    a,
                    b,
                    tolerance,
                    (a.0 - tolerance, a.1 - tolerance, 1.2 - tolerance),
                    (a.0 + tolerance, a.1 + tolerance, 1.2 + tolerance),
                    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
                )
            }
        }

        // Darkish cyan
        assert_close(pixel_rgb_to_hsv(0.438, 0.875, 0.875), (180.0, 0.5, 0.875));

        // Pinkish black
        assert_close(pixel_rgb_to_hsv(0.25, 0.125, 0.125), (0.0, 0.5, 0.25));
    }

    // Simple colors. Maxed Red, Green, and Blue
    #[test]
    fn hsv_to_rgb_simple() {
        // White. Every color maxed
        assert_eq!(pixel_hsv_to_rgb(0.0, 0.0, 1.0), (1.0, 1.0, 1.0));

        assert_eq!(pixel_hsv_to_rgb(0.0, 1.0, 1.0), (1.0, 0.0, 0.0));

        assert_eq!(pixel_hsv_to_rgb(120.0, 1.0, 1.0), (0.0, 1.0, 0.0));

        assert_eq!(pixel_hsv_to_rgb(240.0, 1.0, 1.0), (0.0, 0.0, 1.0));
    }
}
