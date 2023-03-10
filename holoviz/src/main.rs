#![feature(test)]
use std::path::PathBuf;

use svg::node::element::path::{Command, Data};
use svg::node::element::tag;
use svg::node::element::{Animate, Circle, Path, Style, SVG};
use svg::parser::Event;
use svg::Document;

mod cli;
mod visualizer;
use clap::Parser;

use visualizer::{Point, Visualizer};

fn main() {
    let args = cli::Args::parse();

    let viz = Visualizer::from_file(args.input_svg).expect("test SVG file not found");

    let hologram = viz.build_static_hologram();

    // let output_file = PathBuf::from("tests/test4-output.svg");

    svg::save(args.output_svg, &hologram).unwrap();
}
/*
fn animate_hologram_single_svg(
    input_file: PathBuf,
    output_file: PathBuf,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file)?;
    let (input_circles, extents) = parse_circles_with_extents(&svg_contents);

    let lss = Point {
        x: DEFAULT_WIDTH_PX * ls_min,
        y: -ly,
    };
    let lse = Point {
        x: DEFAULT_HEIGHT_PX * ls_max,
        y: -ly,
    };

    let mut viewbox = SVG::new().set("viewBox", extents.clone().as_tuple());
    let style = Style::new(include_str!("../style.css"));
    for circle in input_circles {
        let new_circle = circle
            .clone()
            .set("class", "inputCircle")
            .set("stroke-width", (extents.width) * CIRCLE_STROKE_WIDTH);
        viewbox = viewbox.add(new_circle);
        // TODO: Rearrange arcs/circles so that arcs are always on top of circles
        // Make option for circles to not be drawn.
        let svg_arc = animated_arc(&circle, &lss, &lse, 2.0)
            .set("class", "outputArc")
            .set("stroke-width", (extents.width) * HOLO_STROKE_WIDTH);
        viewbox = viewbox.add(svg_arc);
    }
    let document = Document::new()
        .set("width", DEFAULT_WIDTH_PX)
        .set("height", DEFAULT_HEIGHT_PX)
        .add(style)
        .add(viewbox);
    svg::save(output_file, &document)?;
    Ok(())
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
fn read_svg(filename: PathBuf) -> Result<String, std::io::Error> {
    let mut content = String::new();
    svg::open(filename, &mut content)?;
    Ok(content)
}

/// Given path data in the form of commands, return a string
/// as would be represented in a rendered SVG file.
/// ```
/// let data = Data::new()
///     .move_to((0, 0))
///     .elliptical_arc_to((80, 80, 0, 0, 0, 10, 10));
/// assert_eq!(data_to_string(&data), "M0 0 A80 80 0 0 0 10 10")
/// ```
/// This function only works for data with Move and Elliptical Arc commands.
/// Any other commands in the path data will cause an unimplemented! failure.
fn data_to_string(data: &Data) -> String {
    let mut result = String::new();
    for command in data.iter() {
        let params = match command {
            Command::Move(_, params) => {
                result.push('M');
                params
            }
            Command::EllipticalArc(_, params) => {
                result.push('A');
                params
            }
            _ => unimplemented!(),
        };
        for param in params.iter() {
            result = result + &format!("{} ", param);
        }
    }
    result.trim().to_string()
}

/// Given an input circle and starting & ending positions for a light
/// source, return a Path object with an animated SVG arc representing
/// a hologram of the light moving back and forth between the two
/// points. Animation has `duration_secs` and will repeat indefinitely.
///
/// TODO: Refactor this to calculate the appropriate number of steps
/// to animate. Current animation only uses start & end positions of light
/// source. A cleaner animation will result from taking intermediate
/// points if the movement of the light source is large.
fn animated_arc(
    input_circle: &Circle,
    light_source_start: &Point,
    light_source_end: &Point,
    duration_secs: f32,
) -> Path {
    assert!(duration_secs > 0f32);
    let circle_attrs = input_circle.get_attributes();
    let cx = circle_attrs["cx"]
        .parse::<f32>()
        .expect("Circle should have an x-coordinate");
    let cy = circle_attrs["cy"]
        .parse::<f32>()
        .expect("Circle should have a y-coordinate");
    let r = circle_attrs["r"]
        .parse::<f32>()
        .expect("Circle should have a radius");
    let frame_start = circular_arc_hologram_path(cx, cy, r, HOLO_WIDTH_DEG, light_source_start);
    let frame_end = circular_arc_hologram_path(cx, cy, r, HOLO_WIDTH_DEG, light_source_end);
    let animation_data: String = [
        data_to_string(&frame_start),
        data_to_string(&frame_end),
        data_to_string(&frame_start),
    ]
    .join(";");
    let animated_arc = Path::new().add(
        Animate::new()
            .set("dur", duration_secs)
            .set("repeatCount", "indefinite")
            .set("attributeName", "d")
            .set("values", animation_data),
    );

    animated_arc
}

*/

/*
#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn build_static_hologram(b: &mut Bencher) {
        let svg_contents =
            read_svg(PathBuf::from("tests/icosahedron.svg")).expect("valid input file");
        let (input_circles, extents) = parse_circles_with_extents(&svg_contents);
        let ls = Point {
            x: extents.width * 0.75,
            y: -100.0,
        };
        b.iter(|| {
            build_hologram(
                &input_circles,
                extents.as_tuple(),
                &ls,
                PathBuf::from("tests/benchmark_out.svg"),
            )
        })
    }

    #[bench]
    fn build_animated_hologram(b: &mut Bencher) {
        b.iter(|| {
            animate_hologram_single_svg(
                PathBuf::from("tests/icosahedron.svg"),
                PathBuf::from("tests/benchmark_out.svg"),
                0.35,
                0.65,
                100.0,
            )
        })
    }

    #[test]
    fn test_data_to_string() {
        let data = Data::new()
            .move_to((0, 0))
            .elliptical_arc_to((80, 80, 0, 0, 0, 10, 10));
        assert_eq!(data_to_string(&data), "M0 0 A80 80 0 0 0 10 10")
    }

    #[test]
    fn test_parse_circles_extents() {
        let svg_content = String::from(
            r#"
<svg height="500" width="500" xmlns="http://www.w3.org/2000/svg">
<svg viewBox="-1.2759765,-1.2759765 2.551953,2.551953" xmlns="http://www.w3.org/2000/svg">
<circle cx="0.850651" cy="0" fill="none" r="-0.13143276" stroke="black" stroke-width="0.005"/>
</svg></svg>
            "#,
        );
        let (circles, extents) = parse_circles_with_extents(&svg_content);
        assert_eq!(
            extents,
            Extents {
                xmin: -1.2759765,
                ymin: -1.2759765,
                width: 2.551953,
                height: 2.551953
            }
        );
        let expected_circle_attrs = circles[0].get_attributes();
        assert_eq!(
            expected_circle_attrs["cx"].parse::<f32>().unwrap(),
            0.850651
        );
        assert_eq!(expected_circle_attrs["cy"].parse::<f32>().unwrap(), 0f32);
        assert_eq!(
            expected_circle_attrs["r"].parse::<f32>().unwrap(),
            -0.13143276
        );
    }

    #[test]
    fn test_parse_extents() {
        let svg_with_viewbox = String::from(
            r#"
<svg
  height="100" width="30"
  viewBox="-10 -20 300 100"
  xmlns="http://www.w3.org/2000/svg"
  stroke="red"
  fill="grey">
  <circle cx="50" cy="50" r="40" />
</svg>
        "#,
        );
        assert_eq!(
            parse_circles_with_extents(&svg_with_viewbox).1.as_tuple(),
            (-10., -20., 300., 100.)
        );
        let svg_without_viewbox = String::from(
            r#"
<svg
  height="100" width="30"
  xmlns="http://www.w3.org/2000/svg"
  fill="grey">
  <circle cx="50" cy="50" r="40" />
</svg>
        "#,
        );
        assert_eq!(
            parse_circles_with_extents(&svg_without_viewbox)
                .1
                .as_tuple(),
            (0., 0., 30., 100.)
        );
        assert_eq!(
            // empty SVG should still get a result
            parse_circles_with_extents(&String::from("<svg/>"))
                .1
                .as_tuple(),
            (0., 0., DEFAULT_WIDTH_PX, DEFAULT_HEIGHT_PX)
        );
    }

    /* "INTEGRATION TESTS" */

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
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// top level SVG. Generates single .svg file in the /tests folder.
    /// Recommend manually examining the output to ensure correctness.
    fn test_icosahedron_single() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/icosahedron.svg");
        let output_handle = PathBuf::from("tests/icosahedron-anim.svg");
        animate_hologram_single_svg(input_file, output_handle, 0.35, 0.65, 100.0)?;
        Ok(())
    }
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// second level SVG, and all circles part of the interior viewbox.
    /// Generates single .svg file in the /tests folder
    /// Recommend manually examining the output to ensure correctness.
    fn test_nested_viewbox_single() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/test4.svg");
        let output_handle = PathBuf::from("tests/test4-anim.svg");
        animate_hologram_single_svg(input_file, output_handle, 0.35, 0.65, 100.0)?;
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
    /// This tests a simple rectangle. There is no viewBox definition
    /// in the input file, only width and height.
    /// Recommend manually examining the output to ensure correctness.
    fn test_no_viewbox_single() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/rectangle.svg");
        let output_handle = PathBuf::from("tests/rect-anim.svg");
        animate_hologram_single_svg(input_file, output_handle, 0.35, 0.65, 100.0)?;
        Ok(())
    }
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
}
*/
