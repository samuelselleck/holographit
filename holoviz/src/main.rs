use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::Document;

fn main() {
    let c = Circle::new()
        .set("cx", 150.0)
        .set("cy", 100.0)
        .set("r", 80.0)
        .set("stroke-width", 1)
        .set("stroke", "white")
        .set("fill-opacity", 0);
    let svg_arc = circular_arc(&c, 5.0, 275.0)
        .set("stroke", "red")
        .set("stroke-width", 3);
    println!("{svg_arc:?}");
    let document = Document::new()
        .set("viewBox", (0, 0, 300, 200))
        .add(c)
        .add(svg_arc);
    svg::save("image.svg", &document).unwrap();
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
