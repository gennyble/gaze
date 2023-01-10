use image::{io::Reader, GenericImageView, Pixel, Rgb};

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);

fn main() {
    let imgname = match std::env::args().nth(1) {
        Some(s) => s,
        None => {
            eprintln!("usage: img2curve <img.png>");
            return;
        }
    };

    let img = Reader::open(imgname).unwrap().decode().unwrap();
    let normal = |point: u32| 1.0 - (point as f32 / (img.height() as f32 - 1.0));

    for x in 0..(img.width() / 2) {
        for y in (0..img.height()).rev() {
            if img.get_pixel(x, y).to_rgb() == BLACK {
                println!("{}", normal(y));
                break;
            }
        }
    }

    for x in img.width() / 2..img.width() {
        for y in 0..img.height() {
            if img.get_pixel(x, y).to_rgb() == BLACK {
                println!("{}", normal(y));
                break;
            }
        }
    }
}
