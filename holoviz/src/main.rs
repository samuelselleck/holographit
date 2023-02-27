struct Circle {
    x: f32,
    y: f32,
    r: f32,
}

#[derive(Debug)]
struct SVGArc {
    x0: f32,         // x start point
    y0: f32,         // y start point
    rx: f32,         // x radius
    ry: f32,         // y radius
    x: f32,          // x end point
    y: f32,          // y end point
    x_rot: f32,      // rotation
    large_arc: bool, // large or small arc
    sweep: bool,     // positive or negative sweep
}

impl SVGArc {
    fn to_svg_path(&self, path_params: &str) -> String {
        format!(
            "<path d=\"M {} {} A {} {} {} {} {} {} {}\" {}/>",
            self.x0,
            self.y0,
            self.rx,
            self.ry,
            self.x_rot,
            self.large_arc as u8,
            self.sweep as u8,
            self.x,
            self.y,
            path_params,
        )
        .to_string()
    }
}

fn main() {
    println!("Hello, world!");
    let c = Circle {
        x: 150.0,
        y: 100.0,
        r: 80.0,
    };
    let svg_arc = circular_arc(c, 5.0, 315.0);
    println!("{svg_arc:?}");
    let path_params = "stroke=\"red\" stroke-width=\"2\"";
    println!("{}", svg_arc.to_svg_path(path_params));
}

/// Given a circle, a cone angle, and an incidence angle, return
/// a SVGArc. The cone_angle represents the width of the arc in degrees;
/// A cone angle of 360 will simply return a full circle.
/// The incidence angle represents where on the circle the center
/// of the arc will be. An angle of 0 deg is at +X (right of screen),
/// 90 deg at +Y (top of screen) etc.
fn circular_arc(circle: Circle, half_cone_angle: f32, incidence_angle: f32) -> SVGArc {
    let x0 = circle.x + circle.r * (incidence_angle - half_cone_angle).to_radians().cos();
    let y0 = circle.y - circle.r * (incidence_angle - half_cone_angle).to_radians().sin();
    let x = circle.x + circle.r * (incidence_angle + half_cone_angle).to_radians().cos();
    let y = circle.y - circle.r * (incidence_angle + half_cone_angle).to_radians().sin();
    SVGArc {
        x0,
        y0,
        rx: circle.r,
        ry: circle.r,
        x,
        y,
        x_rot: 0.0,
        large_arc: false,
        sweep: true,
    }
}
