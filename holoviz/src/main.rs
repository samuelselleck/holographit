use svg::node::element::path::Data;
use svg::node::element::{Circle, Path, SVG};
use svg::parser::Event;
use svg::Document;

mod cli;
use clap::Parser;
const HOLO_WIDTH_DEG: f32 = 3.5;
// distance of light source from top of SVG canvas
// Light source is assumed to be centered above canvas for now

const DEFAULT_WIDTH: f32 = 500.;
const DEFAULT_HEIGHT: f32 = 500.;

const HOLO_WIDTH: f32 = 0.005;
const CIRCLE_WIDTH: f32 = 0.0001;

struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let args = cli::Args::parse();

    animate_hologram(
        &args.input_svg,
        &args.output_svg,
        args.num_steps,
        args.lxmin,
        args.lxmax,
        args.ly,
    )
    .unwrap();
    // After running this, run the following:
    // ffmpeg -f image2 -framerate 15 -i <output_svg>%03d.svg output.gif
    // build_hologram(&input_circles, extents, &light_source, args.output_svg);
}

/// Make an animated hologram based on curves in an input file.
/// This function generates num_steps * 2 .svg files, which are intended
/// to be stiched together into an animated .gif using a command such as
/// `ffmpeg -f image2 -framerate 15 -i <output_svg>%03d.svg output.gif`
/// The animation will reverse to make a complete loop.
///
/// ls_min and ls_max are the minimum and maximum locations of the light
/// source relative to the width of the canvas, respectively.
fn animate_hologram(
    input_file: &str,
    output_handle: &str,
    num_steps: u32,
    ls_min: f32,
    ls_max: f32,
    ly: f32,
) -> Result<(), std::io::Error> {
    let svg_contents = read_svg(input_file).expect("valid input file");
    let input_circles = parse_svg_circles(&svg_contents);
    let extents = parse_viewbox_extents(&svg_contents);
    let width = extents.2 - extents.0;
    let step_size = width * (ls_max - ls_min) / num_steps as f32;
    println!("Image has width of {width}, using step size of {step_size}");
    for image in 0..num_steps * 2 {
        let lsx = match image < num_steps {
            true => width * ls_min + image as f32 * step_size,
            false => width * ls_max - (image - num_steps) as f32 * step_size,
        };
        let ls = Point { x: lsx, y: -ly };
        let filename = format!("{}{image:03}.svg", output_handle);
        build_hologram(&input_circles, extents, &ls, filename)?;
        // println!("{filename} - {}", ls.x)
    }
    Ok(())
}

fn build_hologram(
    circles: &Vec<Circle>,
    extents: (f32, f32, f32, f32),
    light_source: &Point,
    filename: String,
) -> Result<(), std::io::Error> {
    let mut viewbox = SVG::new().set("viewBox", extents);
    for circle in circles {
        let new_circle = circle
            .clone()
            .set("stroke-width", (extents.2 - extents.0) * CIRCLE_WIDTH)
            .set("stroke", "grey")
            .set("stroke-opacity", 0.5)
            .set("fill-opacity", 0);
        viewbox = viewbox.add(new_circle);
        let svg_arc = arc_from_light_source(&circle, HOLO_WIDTH_DEG, &light_source)
            .set("stroke", "red")
            .set("stroke-width", (extents.2 - extents.0) * HOLO_WIDTH);
        viewbox = viewbox.add(svg_arc);
    }
    let document = Document::new()
        .set("width", DEFAULT_WIDTH)
        .set("height", DEFAULT_HEIGHT)
        .add(viewbox);
    svg::save(filename, &document)?;
    Ok(())
}

fn read_svg(filename: &str) -> Result<String, std::io::Error> {
    let mut content = String::new();
    svg::open(filename, &mut content)?;
    Ok(content)
}

// TODO: Make this function simply return the incidence angle rather than passing
// half_cone_angle through unmodified
fn arc_from_light_source(circle: &Circle, half_cone_angle: f32, light_source: &Point) -> Path {
    let circle_attrs = circle.get_attributes();
    let cx = circle_attrs["cx"]
        .parse::<f32>()
        .expect("Circle should have an x-coordinate");
    let cy = circle_attrs["cy"]
        .parse::<f32>()
        .expect("Circle should have a y-coordinate");
    // let r = circle_attrs["r"]
    //     .parse::<f32>()
    //     .expect("Circle should have a radius");
    let dx = light_source.x - cx;
    let dy = light_source.y - cy;
    let mut theta = (-dy / dx).atan();
    if theta < 0f32 {
        //} std::f32::consts::PI {
        theta -= std::f32::consts::PI;
    }

    circular_arc(circle, half_cone_angle, theta.to_degrees())
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
    println!("{:?}", svg_tag);
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

/// Given a circle, a cone angle, and an incidence angle, return
/// a SVGArc. The cone_angle represents the width of the arc in degrees;
/// A cone angle of 360 will simply return a full circle.
/// The incidence angle represents where on the circle the center
/// of the arc will be. An angle of 0 deg is at +X (right of screen),
/// 90 deg at +Y (top of screen) etc.
fn circular_arc(circle: &Circle, half_cone_angle: f32, incidence_angle: f32) -> Path {
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
    let x0 = cx + r * (incidence_angle - half_cone_angle).to_radians().cos();
    let y0 = cy - r * (incidence_angle - half_cone_angle).to_radians().sin();
    let x = cx + r * (incidence_angle + half_cone_angle).to_radians().cos();
    let y = cy - r * (incidence_angle + half_cone_angle).to_radians().sin();
    let path_data = Data::new()
        .move_to((x0, y0))
        .elliptical_arc_to((r, r, 0, 0, 0, x, y));
    Path::new().set("d", path_data)
}

#[cfg(test)]
mod tests {
    use super::*;

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
