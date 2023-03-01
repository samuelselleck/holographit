use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::parser::Event;
use svg::Document;

mod cli;
use clap::Parser;
const HOLO_WIDTH_DEG: f32 = 3.5;
// distance of light source from top of SVG canvas
// Light source is assumed to be centered above canvas for now
const LIGHT_SOURCE_DIST: f32 = 60.0; // TODO: Make parameter

const DEFAULT_WIDTH: f32 = 500.;
const DEFAULT_HEIGHT: f32 = 500.;
struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let args = cli::Args::parse();
    let svg_contents = read_svg(&args.input_svg).expect("valid input file");

    let input_circles = parse_svg_circles(&svg_contents);
    let extents = parse_viewbox_extents(&svg_contents);
    println!("{:?}", extents);
    // TODO: Make size of output canvas same size as input canvas
    let light_source = Point {
        x: args.lx, // TODO: Make optional, default at center of canvas
        y: -LIGHT_SOURCE_DIST,
    };
    // TODO: Break out into separate function
    let mut document = Document::new().set("viewBox", (-20, -20, 41, 41)).clone();
    for circle in input_circles {
        let new_circle = circle
            .clone()
            .set("stroke-width", 0.05)
            .set("stroke", "grey")
            .set("stroke-opacity", 0.05)
            .set("fill-opacity", 0);
        document = document.add(new_circle);
        let svg_arc = arc_from_light_source(&circle, HOLO_WIDTH_DEG, &light_source)
            .set("stroke", "red")
            .set("stroke-width", 0.15);
        document = document.add(svg_arc);
    }
    svg::save(args.output_svg, &document).unwrap();
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
