#![feature(test)]
use std::path::PathBuf;

use svg::node::element::path::{Command, Data};
use svg::node::element::{Animate, Circle, Path, Style, SVG};
use svg::parser::Event;
use svg::Document;

mod cli;
use clap::Parser;

extern crate test;

// width of reflected hologram segments in degrees
const HOLO_WIDTH_DEG: f32 = 3.5;

// default width & height of an output SVG image in pixels
const DEFAULT_WIDTH: f32 = 500.;
const DEFAULT_HEIGHT: f32 = 500.;

// stroke widths are intended to be fractions of the input file's
// viewbox width.
const HOLO_STROKE_WIDTH: f32 = 0.005;
const CIRCLE_STROKE_WIDTH: f32 = 0.0001;

struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let args = cli::Args::parse();

    // let input_file = PathBuf::from("circles.svg");
    // let svg_contents = read_svg(input_file).unwrap();
    // let input_circles = parse_svg_circles(&svg_contents);
    // let style = Style::new(include_str!("../style.css"));

    // let a = animated_arc(
    //     &input_circles[1],
    //     &Point { x: 150., y: -100. },
    //     &Point { x: 250., y: -100. },
    //     3.0,
    //     // 5.0,
    // );

    // let atest = Document::new()
    //     .add(style)
    //     .add(a)
    //     .set("width", DEFAULT_WIDTH)
    //     .set("height", DEFAULT_HEIGHT);
    // svg::save("atest.svg", &atest);
    match animate_hologram_single_svg(
        args.input_svg,
        args.output_svg,
        // args.num_steps,
        args.lxmin,
        args.lxmax,
        args.ly,
    ) {
        Ok(()) => {}
        Err(err) => {
            println!("ERROR: {err:?}")
        }
    }
    // After running this, run the following:
    // ffmpeg -f image2 -framerate 15 -i <output_svg>%03d.svg output.gif
    // build_hologram(&input_circles, extents, &light_source, args.output_svg);
}

fn animate_hologram_single_svg(
    input_file: PathBuf,
    output_file: PathBuf,
    // num_steps: u32,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file)?;
    let input_circles = parse_svg_circles(&svg_contents);
    let extents = parse_viewbox_extents(&svg_contents);
    let width = extents.2 - extents.0;
    // let step_size = width * (ls_max - ls_min) / num_steps as f32;
    let lss = Point {
        x: width * ls_min,
        y: -ly,
    };
    let lse = Point {
        x: width * ls_max,
        y: -ly,
    };

    let mut viewbox = SVG::new().set("viewBox", extents);
    let style = Style::new(include_str!("../style.css"));
    for circle in input_circles {
        let new_circle = circle.clone().set("class", "inputCircle").set(
            "stroke-width",
            (extents.2 - extents.0) * CIRCLE_STROKE_WIDTH,
        );
        viewbox = viewbox.add(new_circle);
        // TODO: Rearrange arcs/circles so that arcs are always on top of circles
        // Make option for circles to not be drawn.
        let svg_arc = animated_arc(&circle, &lss, &lse, 2.0)
            .set("class", "outputArc")
            .set("stroke-width", (extents.2 - extents.0) * HOLO_STROKE_WIDTH);
        viewbox = viewbox.add(svg_arc);
    }
    let document = Document::new()
        .set("width", DEFAULT_WIDTH)
        .set("height", DEFAULT_HEIGHT)
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
fn animate_hologram_multi_svg(
    input_file: PathBuf,
    output_handle: &str,
    num_steps: u32,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file)?;
    let input_circles = parse_svg_circles(&svg_contents);
    let extents = parse_viewbox_extents(&svg_contents);
    let width = extents.2 - extents.0;
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
        let mut filename: PathBuf = PathBuf::from(format!("{}{image:03}", output_handle));
        filename.set_extension("svg");
        // TODO: Don't build holograms that we've already built when
        // reversing the animation!
        build_hologram(&input_circles, extents, &ls, filename)?;
        // println!("{filename} - {}", ls.x)
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
        .set("width", DEFAULT_WIDTH)
        .set("height", DEFAULT_HEIGHT)
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
    result
}
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
    let frame_1 = circular_arc_hologram_path(cx, cy, r, HOLO_WIDTH_DEG, light_source_start);
    let frame_2 = circular_arc_hologram_path(cx, cy, r, HOLO_WIDTH_DEG, light_source_end);
    let animation_data: String = [
        data_to_string(&frame_1),
        data_to_string(&frame_2),
        data_to_string(&frame_1),
    ]
    .join(";");
    let animated_arc = Path::new()
        // .set("class", "outputArc")
        // .set("stroke-width", stroke_width)
        .add(
            Animate::new()
                .set("dur", duration_secs)
                .set("repeatCount", "indefinite")
                .set("attributeName", "d")
                .set("values", animation_data),
        );

    animated_arc
}

fn circular_arc_hologram_path(
    cx: f32,
    cy: f32,
    r: f32,
    half_cone_angle: f32,
    light_source: &Point,
) -> Data {
    let dx = light_source.x - cx;
    let dy = light_source.y - cy;
    let mut theta = (-dy / dx).atan();
    if theta < 0f32 {
        theta -= std::f32::consts::PI;
    }
    let x0 = cx + r * (theta - half_cone_angle.to_radians()).cos();
    let y0 = cy - r * (theta - half_cone_angle.to_radians()).sin();
    let x = cx + r * (theta + half_cone_angle.to_radians()).cos();
    let y = cy - r * (theta + half_cone_angle.to_radians()).sin();
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

/// Given the contents of an SVG file, determine the extents of the drawing
/// to be used in lightsource application. Returns a 4-tuple of floats in the
/// form (xmin, ymin, xmax, ymax). If the viewBox attribute has been set in the top-level
/// <svg> tag, the extents will be equal to the viewBox. Otherwise the
/// xmin and ymin values will be set to zero, and xmax and ymax will be set
/// to the "width" and "height" parameters from the top-level <svg> tag. If these
/// are now found, we resort to defaults. Panics if no top level svg tag found.
fn parse_viewbox_extents(svg_contents: &String) -> (f32, f32, f32, f32) {
    let svg_tag = svg::Parser::new(&svg_contents)
        .next()
        .expect("empty svg file");
    // println!("{:?}", svg_tag);
    let extents: Vec<f32> = match svg_tag {
        Event::Tag("svg", _, attributes) => {
            if let Some(view_box) = attributes.get("viewBox") {
                view_box
                    .clone()
                    .split(' ') // TODO: Handle svgs that use comma-separated values
                    .map(|b| b.parse::<f32>().expect("invalid view bounds!"))
                    .collect()
            } else {
                let width = match attributes.get("width") {
                    Some(w) => w.parse::<f32>().expect("invalid width!"),
                    None => DEFAULT_WIDTH,
                };
                let height = match attributes.get("height") {
                    Some(h) => h.parse::<f32>().expect("invalid height!"),
                    None => DEFAULT_HEIGHT,
                };
                vec![0., 0., width, height]
            }
        }
        _ => {
            // println!("{:?}", svg_tag);
            panic!("file does not open with <svg> tag with viewBox value!");
        }
    };
    assert_eq!(extents.len(), 4, "invalid number of view bounds");
    // TODO: Clean up, make a nicer way to build this tuple
    (extents[0], extents[1], extents[2], extents[3])
}

/// Open an SVG file and return a vector of all circles found in that file.
/// This function only returns geometric definitions (cx, cy, and r), and drops
/// any non-geometric attributes such as stroke, fill, color, etc.
fn parse_svg_circles(svg_contents: &String) -> Vec<Circle> {
    let parser = svg::Parser::new(&svg_contents);
    let mut circles = vec![];
    for event in parser {
        match event {
            Event::Tag("circle", _, attributes) => {
                // println!(
                //     "Found a circle at x={}",
                //     attributes.get("cx").expect("should have x-coord")
                // );
                let new_circle = Circle::new()
                    .set("cx", attributes["cx"].clone())
                    .set("cy", attributes["cy"].clone())
                    .set("r", attributes["r"].clone());
                circles.push(new_circle);
            }
            _ => {
                // println!("{:?}", event);
                // println!("Sorry I can only do circles rn");
                continue;
            }
        }
    }
    circles
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn build_static_hologram(b: &mut Bencher) {
        let svg_contents =
            read_svg(PathBuf::from("tests/icosahedron.svg")).expect("valid input file");
        let input_circles = parse_svg_circles(&svg_contents);
        let extents = parse_viewbox_extents(&svg_contents);
        let width = extents.2 - extents.0;
        let ls = Point {
            x: width * 0.75,
            y: -100.0,
        };
        b.iter(|| {
            build_hologram(
                &input_circles,
                extents,
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
            parse_viewbox_extents(&svg_with_viewbox),
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
            parse_viewbox_extents(&svg_without_viewbox),
            (0., 0., 30., 100.)
        );
        assert_eq!(
            // empty SVG should still get a result
            parse_viewbox_extents(&String::from("<svg/>")),
            (0., 0., DEFAULT_WIDTH, DEFAULT_HEIGHT)
        );
    }

    #[test]
    #[should_panic(expected = "file does not open with <svg> tag with viewBox value!")]
    fn test_parse_no_svg_tag() {
        parse_viewbox_extents(&String::from("<html></html>"));
    }

    #[test]
    #[should_panic(expected = "empty svg file")]
    fn test_parse_empty_svg() {
        parse_viewbox_extents(&String::new());
    }
}
