use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Input file: expects a .svg with circles
    pub input_svg: PathBuf,

    /// Output file: expects .svg
    pub output_svg: PathBuf,

    /// Animation duration in seconds
    #[arg(default_value_t = 2.)]
    pub duration: f32,

    /// Animate the output
    #[arg(short, long)]
    pub animate: bool,
}
