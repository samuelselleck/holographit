#![feature(test)]
use std::path::PathBuf;

use svg::node::element::path::{Command, Data};
use svg::node::element::tag;
use svg::node::element::{Animate, Circle, Path, Style, SVG};
use svg::parser::Event;
use svg::Document;

mod cli;
use clap::Parser;

extern crate test;

// width of reflected hologram segments in degrees
const HOLO_WIDTH_DEG: f32 = 3.5;

// default width & height of an output SVG image in pixels
const DEFAULT_WIDTH_PX: f32 = 500.;
const DEFAULT_HEIGHT_PX: f32 = 500.;

// stroke widths are intended to be fractions of the input file's
// viewbox width.
const HOLO_STROKE_WIDTH: f32 = 0.005;
const CIRCLE_STROKE_WIDTH: f32 = 0.0001;

struct Point {
    x: f32,
    y: f32,
}

/// SVG viewBox extents
#[derive(PartialEq, Debug, Copy, Clone)]
struct Extents {
    xmin: f32,
    ymin: f32,
    xmax: f32,
    ymax: f32,
}

impl Extents {
    fn as_tuple(self) -> (f32, f32, f32, f32) {
        (self.xmin, self.ymin, self.xmax, self.ymax)
    }

    fn from_vec(vec: Vec<f32>) -> Self {
        // TODO: Error handling if vec is not the right size.
        Extents {
            xmin: vec[0],
            ymin: vec[1],
            xmax: vec[2],
            ymax: vec[3],
        }
    }
}
fn main() {
    let args = cli::Args::parse();

    match args.multi_svg {
        true => {
            println!("Generating Multi-SVG Animation");
            match animate_hologram_multi_svg(
                args.input_svg,
                args.output_svg,
                args.num_steps,
                args.lxmin,
                args.lxmax,
                args.ly,
            ) {
                Ok(()) => {}
                Err(err) => {
                    println!("ERROR: {err:?}")
                }
            }
        }
        false => {
            println!("Generating Single SVG...");
            match animate_hologram_single_svg(
                args.input_svg,
                args.output_svg,
                args.lxmin,
                args.lxmax,
                args.ly,
            ) {
                Ok(()) => {}
                Err(err) => {
                    println!("ERROR: {err:?}")
                }
            }
        }
    }
}

fn animate_hologram_single_svg(
    input_file: PathBuf,
    output_file: PathBuf,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file)?;
    let (input_circles, extents) = parse_circles_with_extents(&svg_contents);
    let width = extents.xmax - extents.xmin;

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
            .set("stroke-width", (width) * CIRCLE_STROKE_WIDTH);
        viewbox = viewbox.add(new_circle);
        // TODO: Rearrange arcs/circles so that arcs are always on top of circles
        // Make option for circles to not be drawn.
        let svg_arc = animated_arc(&circle, &lss, &lse, 2.0)
            .set("class", "outputArc")
            .set("stroke-width", (width) * HOLO_STROKE_WIDTH);
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
    let width = extents.xmax - extents.xmin;
    let step_size = width * (ls_max - ls_min) / num_steps as f32;
    println!("Image has width of {width}, using step size of {step_size}");
    // TODO: update this part of the code so that num_steps is the
    // _total_ number of frames. Currently returns 2X the number of frames
    // as the reversal is also computed
    for image in 0..num_steps * 2 {
        let lsx = match image < num_steps {
            true => width * ls_min + image as f32 * step_size,
            false => width * ls_max - (image - num_steps) as f32 * step_size,
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

/// Given one or more circles in an SVG file, the extents of the
/// viewBox in the input file and a lightsource, make a new SVG
/// called filename that has the input circles in light grey, and the
/// reflected portions highlighted in red.
fn build_hologram(
    circles: &Vec<Circle>,
    extents: (f32, f32, f32, f32),
    light_source: &Point,
    filename: PathBuf,
) -> Result<(), std::io::Error> {
    let mut viewbox = SVG::new().set("viewBox", extents);
    let style = Style::new(include_str!("../style.css"));
    for circle in circles {
        let new_circle = circle.clone().set("class", "inputCircle").set(
            "stroke-width",
            (extents.2 - extents.0) * CIRCLE_STROKE_WIDTH,
        );
        viewbox = viewbox.add(new_circle);
        // TODO: Rearrange arcs/circles so that arcs are always on top of circles
        // Make option for circles to not be drawn.
        let svg_arc = arc_from_light_source(&circle, HOLO_WIDTH_DEG, &light_source)
            .set("class", "outputArc")
            .set("stroke-width", (extents.2 - extents.0) * HOLO_STROKE_WIDTH);
        viewbox = viewbox.add(svg_arc);
    }
    let document = Document::new()
        .set("width", DEFAULT_WIDTH_PX)
        .set("height", DEFAULT_HEIGHT_PX)
        .add(style)
        .add(viewbox);
    svg::save(filename, &document)?;
    Ok(())
}

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

/// Given circle parameters, a light source point, and a half-cone angle,
/// generate a circular arc Data object representing the reflected portion
/// of the hologram.
fn circular_arc_hologram_path(
    cx: f32,
    cy: f32,
    r: f32, // TODO: validate that r > 0 ?
    half_cone_angle_deg: f32,
    light_source: &Point,
) -> Data {
    let dx = light_source.x - cx;
    let dy = light_source.y - cy;
    let mut incidence_angle = (-dy / dx).atan();
    if incidence_angle < 0f32 {
        incidence_angle -= std::f32::consts::PI;
    }
    let half_cone_angle_rad = half_cone_angle_deg.to_radians();
    let x0 = cx + r * (incidence_angle - half_cone_angle_rad).cos();
    let y0 = cy - r * (incidence_angle - half_cone_angle_rad).sin();
    let x = cx + r * (incidence_angle + half_cone_angle_rad).cos();
    let y = cy - r * (incidence_angle + half_cone_angle_rad).sin();
    let path_data = Data::new()
        .move_to((x0, y0))
        .elliptical_arc_to((r, r, 0, 0, 0, x, y));

    path_data
}
/// Given a circle, a light source, and a half-cone angle, return a Path
/// that represents a circular arc about the point on the circle that is
/// normal to the light source.
fn arc_from_light_source(circle: &Circle, half_cone_angle: f32, light_source: &Point) -> Path {
    let circle_attrs = circle.get_attributes();
    let cx = circle_attrs["cx"]
        .parse::<f32>()
        .expect("Circle should have an x-coordinate");
    let cy = circle_attrs["cy"]
        .parse::<f32>()
        .expect("Circle should have a y-coordinate");
    let r = circle_attrs["r"]
        .parse::<f32>()
        .expect("Circle should have a radius");
    Path::new().set(
        "d",
        circular_arc_hologram_path(cx, cy, r, half_cone_angle, light_source),
    )
}

/// Given the contents of an SVG file, return a vector of Circle objects
/// and the extents of the viewBox of which these circles are children.
fn parse_circles_with_extents(svg_contents: &String) -> (Vec<Circle>, Extents) {
    let parser = svg::Parser::new(&svg_contents);
    let mut circles = vec![];
    let mut extents = Extents {
        xmin: 0.,
        ymin: 0.,
        xmax: DEFAULT_WIDTH_PX,
        ymax: DEFAULT_HEIGHT_PX,
    };
    // TODO: Handle empty or invalid SVG files
    for event in parser {
        match event {
            Event::Tag("svg", tag::Type::Start, attributes) => {
                let extent_vec;
                if let Some(view_box) = attributes.get("viewBox") {
                    extent_vec = view_box
                        .clone()
                        .split([' ', ','])
                        .map(|b| b.parse::<f32>().expect("invalid view bounds!"))
                        .collect();
                } else {
                    let width = match attributes.get("width") {
                        Some(w) => w.parse::<f32>().expect("invalid width!"),
                        None => DEFAULT_WIDTH_PX,
                    };
                    let height = match attributes.get("height") {
                        Some(h) => h.parse::<f32>().expect("invalid height!"),
                        None => DEFAULT_HEIGHT_PX,
                    };
                    extent_vec = vec![0., 0., width, height];
                }
                extents = Extents::from_vec(extent_vec);
            }
            Event::Tag("circle", _, attributes) => {
                let new_circle = Circle::new()
                    .set("cx", attributes["cx"].clone())
                    .set("cy", attributes["cy"].clone())
                    .set("r", attributes["r"].clone());
                circles.push(new_circle);
            }
            Event::Tag("svg", tag::Type::End, _) => {
                break;
            }
            _ => {
                println!("Warning: Unknown SVG tag in input!");
                continue;
            }
        }
    }
    (circles, extents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn build_static_hologram(b: &mut Bencher) {
        let svg_contents =
            read_svg(PathBuf::from("tests/icosahedron.svg")).expect("valid input file");
        let (input_circles, extents) = parse_circles_with_extents(&svg_contents);
        let width = extents.xmax - extents.xmin;
        let ls = Point {
            x: width * 0.75,
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
                xmax: 2.551953,
                ymax: 2.551953
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

    #[test]
    fn test_icosahedron_multi() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/icosahedron.svg");
        let output_handle = PathBuf::from("tests/icosahedron");
        animate_hologram_multi_svg(input_file, output_handle, 10, 0.35, 0.65, 100.0)?;
        Ok(())
    }
    #[test]
    fn test_icosahedron_single() -> Result<(), std::io::Error> {
        let input_file = PathBuf::from("tests/icosahedron.svg");
        let output_handle = PathBuf::from("tests/icosahedron-anim.svg");
        animate_hologram_single_svg(input_file, output_handle, 0.35, 0.65, 100.0)?;
        Ok(())
    }
}
