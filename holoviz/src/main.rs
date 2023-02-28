use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::parser::Event;
use svg::Document;

const VIEW_ANGLE_DEG: f32 = 85.;
const HOLOD_WIDTH_DEG: f32 = 3.5;

fn main() {
    let input_circles = parse_svg_circles("circles.svg").expect("valid input file");
    let mut document = Document::new().set("viewBox", (0, 0, 300, 200)).clone();
    for circle in input_circles {
        let new_circle = circle
            .clone()
            .set("stroke-width", 1)
            .set("stroke", "grey")
            .set("stroke-opacity", 0.5)
            .set("fill-opacity", 0);
        document = document.add(new_circle);
        let svg_arc = circular_arc(&circle, HOLOD_WIDTH_DEG, VIEW_ANGLE_DEG)
            .set("stroke", "red")
            .set("stroke-width", 3);
        document = document.add(svg_arc);
    }
    svg::save("image.svg", &document).unwrap();
}

/// Open an SVG file and return a vector of all circles found in that file.
/// This function only returns geometric definitions (cx, cy, and r), and drops
/// any non-geometric attributes such as stroke, fill, color, etc.
fn parse_svg_circles(filename: &str) -> Result<Vec<Circle>, std::io::Error> {
    let mut content = String::new();
    let parser = svg::open(filename, &mut content)?;
    let mut circles = vec![];
    for event in parser {
        match event {
            Event::Tag("circle", _, attributes) => {
                println!(
                    "Found a circle at x={}",
                    attributes.get("cx").expect("should have x-coord")
                );
                let new_circle = Circle::new()
                    .set("cx", attributes["cx"].clone())
                    .set("cy", attributes["cy"].clone())
                    .set("r", attributes["r"].clone());
                circles.push(new_circle);
            }
            _ => {
                // println!("Sorry I can only do circles rn");
                continue;
            }
        }
    }
    Ok(circles)
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
