#![feature(test)]

mod cli;
mod visualizer;
use clap::Parser;

use visualizer::{Point, Visualizer};

fn main() {
    let args = cli::Args::parse();

    let viz = Visualizer::from_file(args.input_svg).expect("test SVG file not found");

    let hologram = match args.animate {
        true => {
            let ls_start = Point { x: 300., y: -100. };
            let ls_end = Point { x: 400., y: -10. };
            viz.build_animated_hologram(&ls_start, &ls_end, args.duration)
        }
        false => viz.build_static_hologram(),
    };

    svg::save(args.output_svg, &hologram).unwrap();
}
/*
/// Make an animated hologram based on curves in an input file.
/// This function generates num_steps * 2 .svg files, which are intended
/// to be stiched together into an animated .gif using a command such as
/// `ffmpeg -f image2 -framerate 15 -i <output_svg>%03d.svg output.gif`
/// The animation will reverse to make a complete loop.
///
/// ls_min and ls_max are the minimum and maximum locations of the light
/// source relative to the width of the canvas, respectively.
///
/// After running this, run the following:
/// ```sh
/// ffmpeg -f image2 -framerate 15 -i <output_svg>%03d.svg output.gif
/// ```
/// build_hologram(&input_circles, extents, &light_source, args.output_svg);
fn animate_hologram_multi_svg(
    input_file: PathBuf,
    output_handle: PathBuf,
    num_steps: u32,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file)?;
    let (input_circles, extents) = parse_circles_with_extents(&svg_contents);
    let step_size = DEFAULT_WIDTH_PX * (ls_max - ls_min) / num_steps as f32;
    println!(
        "Image has width of {}, using step size of {}",
        &extents.width, step_size
    );
    // TODO: update this part of the code so that num_steps is the
    // _total_ number of frames. Currently returns 2X the number of frames
    // as the reversal is also computed
    for image in 0..num_steps * 2 {
        let lsx = match image < num_steps {
            true => DEFAULT_WIDTH_PX * ls_min + image as f32 * step_size,
            false => DEFAULT_WIDTH_PX * ls_max - (image - num_steps) as f32 * step_size,
        };
        let ls = Point { x: lsx, y: -ly };
        let mut filename: PathBuf = PathBuf::from(format!(
            "{}{image:03}",
            output_handle.to_str().expect("Non-empty output handle")
        ));
        filename.set_extension("svg");
        // TODO: Don't build holograms that we've already built when
        // reversing the animation!
        build_hologram(&input_circles, extents.as_tuple(), &ls, filename)?;
    }
    Ok(())
}
*/

/*
#[cfg(test)]
mod tests {
    #[test]
    /// This tests a simple rectangle. There is no viewBox definition
    /// in the input file, only width and height.
    /// Generates mltiple .svg files in the /tests folder
    ///
    /// After running this, run the following:
    /// ```sh
    /// ffmpeg -f image2 -framerate 15 -i rect-%03d.svg output.gif
    /// ```
    /// Recommend manually examining the output to ensure correctness.
    fn test_no_viewbox_multi() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/rectangle.svg");
        let output_handle = PathBuf::from("tests/rect-");
        animate_hologram_multi_svg(input_file, output_handle, 10, 0.35, 0.65, 100.0)?;
        Ok(())
    }
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// second level SVG, and all circles part of the interior viewbox.
    /// Generates mltiple .svg files in the /tests folder
    ///
    /// After running this, run the following:
    /// ```sh
    /// ffmpeg -f image2 -framerate 15 -i test4-%03d.svg output.gif
    /// ```
    /// Recommend manually examining the output to ensure correctness.
    fn test_nested_viewbox_multi() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/test4.svg");
        let output_handle = PathBuf::from("tests/test4-");
        animate_hologram_multi_svg(input_file, output_handle, 10, 0.35, 0.65, 100.0)?;
        Ok(())
    }
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// top level SVG. Generates multiple .svg files in the /tests folder
    ///
    /// After running this, run the following:
    /// ```sh
    /// ffmpeg -f image2 -framerate 15 -i icosahedron-%03d.svg output.gif
    /// ```
    /// Recommend manually examining the output to ensure correctness.
    fn test_icosahedron_multi() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/icosahedron.svg");
        let output_handle = PathBuf::from("tests/icosahedron-");
        animate_hologram_multi_svg(input_file, output_handle, 10, 0.35, 0.65, 100.0)?;
        Ok(())
    }
}
*/
