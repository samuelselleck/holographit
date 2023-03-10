use std::path::PathBuf;

use svg::node::element::path::{Command, Data};
use svg::node::element::tag;
use svg::node::element::{Animate, Circle, Path, Style, SVG};
use svg::parser::Event;
use svg::Document;

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

pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// SVG viewBox extents
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Extents {
    pub xmin: f32,
    pub ymin: f32,
    pub width: f32,
    pub height: f32,
}
pub struct Visualizer {
    input_circles: Vec<Circle>,
    extents: Extents,
    light_source: Point,
    style: Style,
    // light_source_path: Path,
    holo_stroke_width: f32,
}

impl Visualizer {
    /// Build a visualizer from a string representing the
    /// contents of an SVG file. Automatically parses the input circles
    /// and extents of the input SVG. Adds a static light source
    /// and a stylesheet.
    pub fn from_svg_contents(contents: String) -> Self {
        let (input_circles, extents) = parse_circles_with_extents(&contents);
        let light_source = Point {
            x: extents.width / 2f32,
            y: -extents.height / 3f32,
        };
        // TODO: Clean this up to use fs::read or something that allows
        // us to use a const declaration earlier in the library (or elsehwere)
        let style = Style::new(include_str!("../style.css"));
        Visualizer {
            input_circles,
            extents,
            light_source,
            style,
            holo_stroke_width: HOLO_STROKE_WIDTH,
        }
    }

    // Build a visualizer from an SVG input file
    pub fn from_file(file_path: PathBuf) -> Result<Self, std::io::Error> {
        let mut content = String::new();
        svg::open(file_path, &mut content)?;
        Ok(Self::from_svg_contents(content))
    }

    /// Build a static hologram from the visualizer
    /// ```
    /// let input_file = PathBuf::from("input_circles.svg")
    /// let viz = Visualizer::from_file(input_file)?;
    /// let hologram = viz.build_static_hologram();
    /// let output_file = PathBuf::from("output_static.svg");
    /// svg::save(output_file, &hologram)?;
    /// ```
    pub fn build_static_hologram(&self) -> Document {
        let mut viewbox = SVG::new().set("viewBox", self.extents.as_tuple());
        for circle in &self.input_circles {
            let new_circle = circle
                .clone()
                .set("class", "inputCircle")
                .set("stroke-width", (self.extents.width) * CIRCLE_STROKE_WIDTH);
            viewbox = viewbox.add(new_circle);
            // TODO: Rearrange arcs/circles so that arcs are always on top of circles
            // Make option for circles to not be drawn.
            let svg_arc = arc_from_light_source(&circle, HOLO_WIDTH_DEG, &self.light_source)
                .set("class", "outputArc")
                .set(
                    "stroke-width",
                    (self.extents.width) * self.holo_stroke_width,
                );
            viewbox = viewbox.add(svg_arc);
        }
        Document::new()
            .set("width", DEFAULT_WIDTH_PX)
            .set("height", DEFAULT_HEIGHT_PX)
            .add(self.style.clone())
            .add(viewbox)
    }

    /// Build an animated hologram from the visualizer. Requires starting
    /// and ending positions of light source relative to the canvas. The
    /// animation will loop back & forth from one light source to the other
    /// indefinitely, with the supplied duration.
    /// ```
    /// let input_file = PathBuf::from("input_circles.svg")
    /// let viz = Visualizer::from_file(input_file)?;
    /// let ls_start = Point { x: 300., y: -100. };
    /// let ls_end = Point { x: 400., y: -50. };
    /// let duration_secs = 2.0;
    /// let hologram = viz.build_animated_hologram(&ls_start, &ls_end, duration_secs);
    /// let output_file = PathBuf::from("output_animated.svg");
    /// svg::save(output_file, &hologram)?;
    /// ```
    pub fn build_animated_hologram(
        &self,
        ls_start: &Point,
        ls_end: &Point,
        duration_secs: f32, // TODO: Input validation, ensure positive number
    ) -> Document {
        let mut viewbox = SVG::new().set("viewBox", self.extents.clone().as_tuple());
        for circle in &self.input_circles {
            let new_circle = circle
                .clone()
                .set("class", "inputCircle")
                .set("stroke-width", (self.extents.width) * CIRCLE_STROKE_WIDTH);
            viewbox = viewbox.add(new_circle);
            // TODO: Rearrange arcs/circles so that arcs are always on top of circles
            // Make option for circles to not be drawn.
            let svg_arc = animated_arc(&circle, &ls_start, &ls_end, duration_secs)
                .set("class", "outputArc")
                .set("stroke-width", (self.extents.width) * HOLO_STROKE_WIDTH);
            viewbox = viewbox.add(svg_arc);
        }
        Document::new()
            .set("width", DEFAULT_WIDTH_PX)
            .set("height", DEFAULT_HEIGHT_PX)
            .add(self.style.clone())
            .add(viewbox)
    }
}

impl Extents {
    fn as_tuple(self) -> (f32, f32, f32, f32) {
        (self.xmin, self.ymin, self.width, self.height)
    }

    fn from_vec(vec: Vec<f32>) -> Self {
        // TODO: Error handling if vec is not the right size.
        Extents {
            xmin: vec[0],
            ymin: vec[1],
            width: vec[2],
            height: vec[3],
        }
    }
}

/// Given the contents of an SVG file, return a vector of Circle objects
/// and the extents of the viewBox of which these circles are children.
fn parse_circles_with_extents(svg_contents: &String) -> (Vec<Circle>, Extents) {
    let parser = svg::Parser::new(&svg_contents);
    let mut circles = vec![];
    let mut extents = Extents {
        xmin: 0.,
        ymin: 0.,
        width: DEFAULT_WIDTH_PX,
        height: DEFAULT_HEIGHT_PX,
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

/*
*/

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

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn build_static_hologram(b: &mut Bencher) {
        let viz = Visualizer::from_file(PathBuf::from("tests/icosahedron.svg"))
            .expect("valid input file");
        b.iter(|| {
            viz.build_static_hologram();
        })
    }

    #[bench]
    fn build_animated_hologram(b: &mut Bencher) {
        let viz = Visualizer::from_file(PathBuf::from("tests/icosahedron.svg"))
            .expect("valid input file");
        let ls_start = Point { x: 300., y: -100. };
        let ls_end = Point { x: 400., y: -10. };
        let duration_secs = 2.0;
        b.iter(|| {
            viz.build_animated_hologram(&ls_start, &ls_end, duration_secs);
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
    fn integration_test(input_path: PathBuf, output_path: PathBuf) -> Result<(), std::io::Error> {
        let viz = Visualizer::from_file(input_path)?;
        let ls_start = Point { x: 300., y: -100. };
        let ls_end = Point { x: 400., y: -50. };
        let duration_secs = 2.0;
        let hologram = viz.build_animated_hologram(&ls_start, &ls_end, duration_secs);
        svg::save(output_path, &hologram).unwrap();

        Ok(())
    }
    #[test]
    /// This test catches an error where the y-coordinate of one of the
    /// light sources is too large, causing two halves of the images to
    /// rotate in different directions.
    fn icosahedron_graphics_error() -> Result<(), std::io::Error> {
        let viz = Visualizer::from_file(PathBuf::from("tests/icosahedron.svg"))?;
        let ls_start = Point { x: 300., y: -100. };
        let ls_end = Point { x: 400., y: 0. };
        let duration_secs = 2.0;
        let hologram = viz.build_animated_hologram(&ls_start, &ls_end, duration_secs);
        svg::save(PathBuf::from("tests/icosahedron-error.svg"), &hologram).unwrap();
        Ok(())
    }
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// top level SVG. Generates single .svg file in the /tests folder.
    /// Recommend manually examining the output to ensure correctness.
    fn test_icosahedron_single() -> Result<(), std::io::Error> {
        let input_path = PathBuf::from("tests/icosahedron.svg");
        let output_path = PathBuf::from("tests/icosahedron-anim.svg");
        integration_test(input_path, output_path)?;
        Ok(())
    }
    #[test]
    /// This tests an icosahedron with the viewbox extents defined at the
    /// second level SVG, and all circles part of the interior viewbox.
    /// Generates single .svg file in the /tests folder
    /// Recommend manually examining the output to ensure correctness.
    fn test_nested_viewbox_single() -> Result<(), std::io::Error> {
        let input_path = PathBuf::from("tests/test4.svg");
        let output_path = PathBuf::from("tests/test4-anim.svg");
        integration_test(input_path, output_path)?;
        Ok(())
    }
    #[test]
    /// This tests a simple rectangle. There is no viewBox definition
    /// in the input file, only width and height.
    /// Recommend manually examining the output to ensure correctness.
    fn test_no_viewbox_single() -> Result<(), std::io::Error> {
        let input_path = PathBuf::from("tests/rectangle.svg");
        let output_path = PathBuf::from("tests/rect-anim.svg");
        integration_test(input_path, output_path)?;
        Ok(())
    }
}
