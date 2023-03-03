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

    /// Y position of light source
    #[arg(default_value_t = 100.)]
    pub ly: f32,

    /// Number of animation steps
    #[arg(default_value_t = 20)]
    pub num_steps: u32,

    /// Minimum X position of light source as a fraction of canvas width
    #[arg(default_value_t = 0.35)]
    pub lxmin: f32,

    /// Maximum X position of light source as a fraction of canvas width
    #[arg(default_value_t = 0.65)]
    pub lxmax: f32,
}