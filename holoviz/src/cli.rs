use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Input file: expects a .svg with circles
    #[arg(short, long)]
    pub input_svg: String,

    /// Output file: expects .svg
    #[arg(short, long)]
    pub output_svg: String,
}
