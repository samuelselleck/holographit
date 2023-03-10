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

const DEFAULT_SYTLESHEET: &str = "../style.css";

pub struct Point {
    x: f32,
    y: f32,
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
    // svg_contents: String,
    input_circles: Vec<Circle>,
    extents: Extents,
    light_source: Point,
    style: Style,
    // light_source_path: Path,
    holo_stroke_width: f32,
}

impl Visualizer {
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
    pub fn from_file(file_path: PathBuf) -> Result<Self, std::io::Error> {
        let mut content = String::new();
        svg::open(file_path, &mut content)?;
        Ok(Self::from_svg_contents(content))
    }

    /// Given one or more circles in an SVG file, the extents of the
    /// viewBox in the input file and a lightsource, make a new SVG
    /// called filename that has the input circles in light grey, and the
    /// reflected portions highlighted in red.
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
                .set("stroke-width", (self.extents.width) * HOLO_STROKE_WIDTH);
            viewbox = viewbox.add(svg_arc);
        }
        let document = Document::new()
            .set("width", DEFAULT_WIDTH_PX)
            .set("height", DEFAULT_HEIGHT_PX)
            .add(self.style.clone())
            .add(viewbox);
        document
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
