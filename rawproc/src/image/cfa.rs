use crate::image::Color;

#[derive(Clone, Debug)]
pub enum CFA {
    /*
    R G R G
    G B G B
    R G R G
    G B G B
    */
    RGGB,
}

impl CFA {
    pub fn color_at(&self, x: u32, y: u32) -> Color {
        match self {
            CFA::RGGB => {
                if x % 2 == 0 {
                    if y % 2 == 0 {
                        return Color::Red;
                    } else {
                        return Color::Green;
                    }
                } else {
                    if y % 2 == 0 {
                        return Color::Green;
                    } else {
                        return Color::Blue;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod cfa_tets {
    use super::*;

    #[test]
    fn color_at_rggb() {
        // Testing initial pattern
        assert_eq!(CFA::RGGB.color_at(0, 0), Color::Red);
        assert_eq!(CFA::RGGB.color_at(1, 0), Color::Green);
        assert_eq!(CFA::RGGB.color_at(0, 1), Color::Green);
        assert_eq!(CFA::RGGB.color_at(1, 1), Color::Blue);

        // Testing expanded pattern
        assert_eq!(CFA::RGGB.color_at(2, 2), Color::Red);
        assert_eq!(CFA::RGGB.color_at(3, 2), Color::Green);
        assert_eq!(CFA::RGGB.color_at(2, 3), Color::Green);
        assert_eq!(CFA::RGGB.color_at(3, 3), Color::Blue);
    }
}
