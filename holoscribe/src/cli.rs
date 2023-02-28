use std::num::ParseIntError;

use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CliError {
    #[error("Invalid output size specification")]
    InvalidSize(ParseIntError),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// A command line utility that take an .obj model and produce a .svg representations of lines to etch
/// to create a holographic image of that model
pub struct Args {
    /// Input file. Expects a .obj file
    #[arg(short, long)]
    pub input: String,

    /// Output file. Expects a .svg file
    #[arg(short, long)]
    pub output: String,

    /// The size in (TODO units) of the image you which to create. Takes the format: width[xheight]
    #[arg(short, long, value_parser=parse_size)]
    pub size: Size,

    /// The density of lines etched per (TODO unit). Defaults to (TODO default)
    #[arg(long, default_value_t = 1)]
    pub stroke_density: usize,
}

#[derive(Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

fn parse_size(arg: &str) -> Result<Size, CliError> {
    if let Some((width, height)) = arg.split_once('x') {
        Ok(Size {
            width: width
                .parse::<usize>()
                .map_err(|e| CliError::InvalidSize(e))?,
            height: height
                .parse::<usize>()
                .map_err(|e| CliError::InvalidSize(e))?,
        })
    } else {
        let width = arg.parse::<usize>().map_err(|e| CliError::InvalidSize(e))?;
        Ok(Size {
            width,
            height: width,
        })
    }
}
