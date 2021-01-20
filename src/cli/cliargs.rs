use super::{OneOrThree, ParseError};
use getopts::Options;
use image::ImageFormat;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct CliArgs {
    pub in_path: PathBuf,
    pub in_is_dir: bool,

    pub out_path: PathBuf,
    pub out_type: ImageFormat,

    pub thumb: bool,
    pub black: Option<OneOrThree<u16>>,
    pub white: Option<OneOrThree<f32>>,
    pub exposure: Option<f32>,
    pub contrast: Option<f32>,
    pub brightness: Option<u8>,
    pub saturation: Option<f32>,
}

impl CliArgs {
    fn usage(program: &str, opts: Options) -> String {
        let brief = format!("Usage: {} FILE [options]", program);
        format!("{}", opts.usage(&brief))
    }

    pub fn new() -> Result<Self, CliError> {
        let cli = Self::from_cli();

        cli
    }

    fn from_cli() -> Result<Self, CliError> {
        let args: Vec<String> = std::env::args().collect();
        let program = &args[0];

        let mut opts = Options::new();
        opts.reqopt(
            "i",
            "ipath",
            "Input path\n\
            If input is a file, the output path is optional.\n\
            If input is a directory, the output path is required",
            "FILE",
        );
        opts.optopt(
            "o",
            "opath",
            "Output path\n
            If no output path is provided, it will default to the input path\
            + the type extension",
            "FILE",
        );
        opts.optopt(
            "",
            "type",
            "Set the output image type\nAvailable types are: png, jpeg",
            "TYPE",
        );
        opts.optflag("t", "thumb", "Scale the image down to 1/4 size");
        opts.optopt(
            "l",
            "black",
            "Black level adjustment values\nDefaults to camera's values\nEx: 150 or 150,200,150",
            "INTS",
        );
        opts.optopt(
            "w",
            "white",
            "White balance adjustment values\nDefaults to camera's values\nEx: 1.0 or 2.1,1.0,1.3",
            "FLOATS",
        );
        opts.optopt("e", "exposure", "Exposure compensation value", "FLOAT");
        opts.optopt("c", "contrast", "Contrast adjustment value", "FLOAT");
        opts.optopt("b", "brightness", "Brightness addition", "INT");
        opts.optopt("s", "saturation", "Saturation scalar", "FLOAT");
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(_e) => {
                return Err(CliError::MatchError(Self::usage(program, opts)));
            }
        };

        let in_path = PathBuf::from(
            matches
                .opt_str("ipath")
                .expect("How'd this happen? ifile isn't present"),
        );
        let in_is_dir = match in_path.metadata() {
            Ok(meta) => meta.is_dir(),
            Err(e) => return Err(CliError::InPathError(e)),
        };

        let mut out_type = if let Some(s) = matches.opt_str("type") {
            match ImageFormat::from_extension(&s) {
                Some(format) => format,
                None => return Err(ParseError::imageformat(s).into()),
            }
        } else {
            // Defaults to jpeg
            ImageFormat::Jpeg
        };

        let out_path = match matches.opt_str("opath").map(|s| PathBuf::from(s)) {
            Some(mut path) => {
                if !in_is_dir {
                    if path.is_dir() {
                        path.push(
                            in_path
                                .file_stem()
                                .expect("File isn't dir but doesn't have stem. How?"),
                        );
                    }

                    match path.extension() {
                        None => {
                            // No extension, add one from out_type
                            path.set_extension(out_type.extensions_str()[0]);
                        }
                        Some(ext) => {
                            // Out path has extension, does it match a format?
                            match ImageFormat::from_extension(ext) {
                                Some(fmt) => {
                                    // Yes! Set the format.
                                    out_type = fmt;
                                }
                                None => {
                                    // No! Return an error
                                    return Err(ParseError::imageformat(
                                        ext.to_str().unwrap().to_owned(),
                                    )
                                    .into());
                                }
                            }
                        }
                    }
                }
                path
            }
            None => {
                if in_is_dir {
                    return Err(CliError::OutPathError);
                } else {
                    let mut out = in_path.clone();
                    out.set_extension(out_type.extensions_str()[0]);
                    out
                }
            }
        };

        let thumb = matches.opt_present("thumb");

        let black = matches.opt_get("black").map_err(|e| ParseError::from(e))?;
        let white = matches.opt_get("white").map_err(|e| ParseError::from(e))?;
        let exposure = matches
            .opt_get("exposure")
            .map_err(|e| ParseError::from(e))?;
        let contrast = matches
            .opt_get("contrast")
            .map_err(|e| ParseError::from(e))?;
        let brightness = matches
            .opt_get("brightness")
            .map_err(|e| ParseError::from(e))?;
        let saturation = matches
            .opt_get("saturation")
            .map_err(|e| ParseError::from(e))?;

        Ok(Self {
            in_path,
            in_is_dir,
            out_path,
            out_type,
            thumb,

            black,
            white,
            exposure,
            contrast,
            brightness,
            saturation,
        })
    }
}

#[derive(Debug)]
pub enum CliError {
    InPathError(IoError),
    OutPathError,
    MatchError(String),
    ParseError(ParseError),
}

//TODO: source
impl Error for CliError {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::InPathError(ioerr) => write!(f, "Failed to open input file: {}", ioerr),
            CliError::OutPathError => write!(
                f,
                "An output path is requried if the input path is a directory\n\
                If you want to output in the current directory, use '.' as the out path"
            ),
            CliError::MatchError(usage) => write!(f, "{}", usage),
            CliError::ParseError(err) => err.fmt(f),
        }
    }
}

impl From<ParseError> for CliError {
    fn from(frm: ParseError) -> Self {
        CliError::ParseError(frm)
    }
}
