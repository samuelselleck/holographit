use std::num::ParseIntError;

use clap::Parser;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CliError {
    #[error("Invalid int specification for size")]
    InvalidSizeInt(ParseIntError),
    #[error("Invalid size specification")]
    InvalidSize,
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

    /// The size of the image you which to create. Requires the format: width[xheight](mm|cm|m)
    #[arg(short, long, value_parser=parse_size)]
    pub size: Size,

    /// The density of lines etched per mm. Defaults to 1.
    #[arg(long, default_value_t = 1)]
    pub stroke_density: usize,
}

/// Represents a size in millimeters
#[derive(Debug, Clone, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

fn parse_size(arg: &str) -> Result<Size, CliError> {
    let re = Regex::new(r"^(\d+)x?(\d*)(mm|cm|m)$").unwrap();
    if let Some(cap) = re.captures(arg) {
        // We always expect to match the first group (width)
        let width_str = cap.get(1).unwrap().as_str();
        let width = width_str
            .parse::<usize>()
            .map_err(|e| CliError::InvalidSizeInt(e))?;

        // The second group (height) is optional, without it just use width
        let height = if let Some(height) = cap.get(2) {
            let height_str = height.as_str();
            if height_str.len() > 0 {
                height
                    .as_str()
                    .parse::<usize>()
                    .map_err(|e| CliError::InvalidSizeInt(e))?
            } else {
                width
            }
        } else {
            width
        };

        // We always expect to have a unit
        let unit = cap.get(3).unwrap().as_str();

        // Normalize to mm
        let factor = match unit {
            "mm" => 1,
            "cm" => 10,
            "m" => 1000,
            _ => panic!("Regex should not have allowed any other unit string"),
        };
        Ok(Size {
            width: width * factor,
            height: height * factor,
        })
    } else {
        Err(CliError::InvalidSize)
    }
}

#[cfg(test)]
mod tests {
    use std::num::{
        IntErrorKind::{self, PosOverflow},
        ParseIntError,
    };

    use crate::cli::{parse_size, CliError, Size};

    #[test]
    fn test_parse_size() {
        // Check that sizes are parsed and units are applied
        assert_eq!(
            parse_size("10x10mm"),
            Ok(Size {
                width: 10,
                height: 10
            })
        );
        assert_eq!(
            parse_size("10x50cm"),
            Ok(Size {
                width: 100,
                height: 500
            })
        );
        assert_eq!(
            parse_size("10cm"),
            Ok(Size {
                width: 100,
                height: 100
            })
        );
        assert_eq!(
            parse_size("2x1m"),
            Ok(Size {
                width: 2000,
                height: 1000
            })
        );

        // Test invalid size specifications
        assert_eq!(parse_size("10x"), Err(CliError::InvalidSize));
        assert_eq!(parse_size("10x10ft"), Err(CliError::InvalidSize));
        assert_eq!(parse_size("-10x10mm"), Err(CliError::InvalidSize));

        // check usize::MAX + 1
        if let CliError::InvalidSizeInt(e) = parse_size("18446744073709552000mm").unwrap_err() {
            let kind = e.kind();
            assert_eq!(kind, &IntErrorKind::PosOverflow);
        } else {
            panic!("Unexpected error type")
        }
    }
}
