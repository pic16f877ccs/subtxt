use clap::{crate_version, value_parser, Arg, ArgMatches, Command, ValueHint};
use image::{open, save_buffer, ColorType, ImageFormat, ImageResult};
use std::error;
use std::fs::{self, write};
use std::path::PathBuf;

type Size = (u32, u32);
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Default)]
struct TxtInImg {
    data: Vec<u8>,
    size: Size,
    rgba: Option<ColorType>,
}

impl TxtInImg {
    fn new() -> TxtInImg {
        TxtInImg::default()
    }

    fn open_image(&mut self, app: &ArgMatches) -> ImageResult<()> {
        if let Some(path) = app
            .get_one::<PathBuf>("input_image")
        {
            let image = open(path)?;
            self.rgba = match image.color() {
                ColorType::Rgba8 => Some(ColorType::Rgba8),
                _ => None,
            };
            self.size = (image.width(), image.height());
            self.data = image.into_rgba8().into_vec();
        }

        Ok(())
    }

    fn encode_data(&mut self, app: &ArgMatches, sub_vec: Vec<u8>) -> Result<()> {
        let mut sub_iter = sub_vec.iter();
        let iter = self.data.chunks_mut(4).filter(|chunk| chunk[3] == 0);

        'outer: for chunk in iter {
            for elem in chunk.iter_mut().take(3) {
                let Some(sub_elem) = sub_iter.next() else {
                    break 'outer;
                };
                *elem = *sub_elem;
            }
        }

        if app.get_flag("ignore") && sub_iter.next().is_some() {
            return Err("there is not enough free space in the image".into());
        }

        Ok(())
    }

    fn write_data(&mut self, app: &ArgMatches, path: &PathBuf) -> Result<()> {
        let mut text_bytes = open_text_file(path)?;
        let mut bytes = encode_text_len(&text_bytes);
        bytes.append(&mut text_bytes);
        self.encode_data(app, bytes)?;
        Ok(())
    }

    fn save_data(&mut self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app
            .get_one::<PathBuf>("input_text")
        {
            let Some(_) = self.rgba else {
                return Err("unsupported color model".into());
            };
            self.write_data(app, &path)?;
        }
        Ok(())
    }

    fn save_img(&self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output") {
            let color_type = ColorType::Rgba8;
            let format = ImageFormat::from_path(path)?;

            if app.contains_id("input_text") {
                let (ImageFormat::Png | ImageFormat::Tiff) = format else {
                    return Err("unsupported image output format".into());
                };
            }

            save_buffer(path, &self.data, self.size.0, self.size.1, color_type)?;
        }
        Ok(())
    }

    fn decode_text_len(&self) -> Option<usize> {
        if self.data.len() <= 12 {
            return None;
        };

        let mut len = Vec::from(&self.data[..10]);
        len.remove(9);
        len.remove(4);

        Some(usize::from_ne_bytes(len.try_into().unwrap()))
    }

    fn decode_text(&mut self) -> Option<Vec<u8>> {
        let Some(len) = self.decode_text_len() else {
            return None;
        };

        let sub_vec = self
            .data
            .chunks_mut(4)
            .filter(|chunk| chunk[3] == 0)
            .flat_map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .skip(12)
            .take(len)
            .collect::<Vec<_>>();

        if sub_vec.len() != len {
            return None;
        }
        Some(sub_vec)
    }

    fn save_invisible_text(&mut self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output_text") {
            let Some(vec) = self.decode_text() else {
                return Err("error extracting text".into());
            };

            write(path, String::from_utf8(vec.to_vec()).unwrap())?;
        }
        Ok(())
    }

    fn print_invisible_text(&mut self, app: &ArgMatches) -> Result<()> {
        if app.get_flag("print") {
            let Some(vec) = self.decode_text() else {
                return Err("error extracting text".into());
            };

            println!("{}\n", String::from_utf8(vec.to_vec()).unwrap());
        }

        Ok(())
    }

    fn print_available_bytes(&self, app: &ArgMatches) {
        if app.get_flag("bytes") {
            if let Some(bytes) = self.available_bytes() {
                println!("\n{} megabytes available in the image\n", bytes / 1_048_576);
            } else {
                println!("\nthere are no available bytes in the image\n");
            }
        }
    }

    fn available_bytes(&self) -> Option<usize> {
        let Some(_) = self.rgba else {
            return None;
        };

        Some(self.data.chunks(4).filter(|chunk| chunk[3] == 0).count() * 3)
    }

    fn alpha_max(&mut self, app: &ArgMatches) {
        if app.get_flag("all") {
            self.data.iter_mut().skip(3).step_by(4).for_each(|alpha| {
                *alpha = 255;
            });
        }
    }
}

fn open_text_file(path: &PathBuf) -> Result<Vec<u8>> {

    Ok(fs::read(path)?)
}

fn encode_text_len(text: &Vec<u8>) -> Vec<u8> {
    let mut vec = Vec::from(text.len().to_ne_bytes());
    vec.insert(3, 0);
    vec.insert(7, 0);
    vec.append(&mut vec![0, 0]);
    vec
}

fn main() -> Result<()> {
    let app = app_commands();
    let mut txt_in_img = TxtInImg::new();
    txt_in_img.open_image(&app)?;
    txt_in_img.print_available_bytes(&app);
    txt_in_img.save_data(&app)?;
    txt_in_img.print_invisible_text(&app)?;
    txt_in_img.save_invisible_text(&app)?;
    txt_in_img.alpha_max(&app);
    txt_in_img.save_img(&app)?;

    Ok(())
}

fn app_commands() -> ArgMatches {
    Command::new("subtxt")
        .about("Tool to hide text using image alpha channel")
        .long_version(crate_version!())
        .author("    by PIC16F877ccs")
        .args_override_self(true)
        .arg(
            Arg::new("input_image")
                .value_name("PAPH")
                .value_parser(value_parser!(PathBuf))
                .index(1)
                .help("Path to input image file")
                .required(true),
        )
        .arg(
            Arg::new("bytes")
                .short('b')
                .long("bytes")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Available bytes in image")
                .required(false),
        )
        .arg(
            Arg::new("input_text")
                .short('i')
                .long("input-text")
                .value_name("PAPH")
                .help("Path to input text file")
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath)
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Make all pixels visible")
                .required(false),
        )
        .arg(
            Arg::new("print")
                .short('p')
                .long("print")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Print the invisible text")
                .required(false),
        )
        .arg(
            Arg::new("ignore")
                .short('I')
                .conflicts_with("output_text")
                .long("ignore")
                .action(clap::ArgAction::SetFalse)
                .num_args(0)
                .help("Ignore text length")
                .required(false),
        )
        .arg(
            Arg::new("output_text")
                .short('O')
                .long("output-text")
                .value_name("PAPH")
                .help("Output text file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PAPH")
                .help("Output image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .get_matches()
}
