use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Input file: expects a .svg with circles
    // #[arg]
    pub input_svg: String,

    /// Output file: expects .svg
    // #[arg]
    pub output_svg: String,

    /// X position of light source
    pub lx: f32,
}
