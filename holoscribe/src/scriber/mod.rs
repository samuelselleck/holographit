use glam::Vec3;
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path, SVG};
use svg::Document;

type Num = f32;

pub struct DebugScriber {
    pub plane_start: Num,
    pub plane_end: Num,
    pub min_size: Num,
    pub max_size: Num,
}

impl DebugScriber {
    fn map_range(&self, z: Num) -> Num {
        self.min_size
            + (z - self.plane_start) * (self.max_size - self.min_size)
                / (self.plane_end - self.plane_start)
    }
}
impl HoloPointStrategy for DebugScriber {
    /// This is a scriber that uses points to construct `Data` to build a `Path` and add it to a `SVG` document. It draws
    /// diamonds around the supplied points
    fn scribe_points(&self, viewbox: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG {
        let data = points.into_iter().fold(Data::new(), |d, &point| {
            let (x, y, z) = (point.x, point.y, point.z);
            let z = self.map_range(z);
            d.move_to((x, y))
                .move_by((0, -z))
                .line_by((-z, z))
                .line_by((z, z))
                .line_by((z, -z))
                .line_by((-z, -z))
        });

        let path = Path::new()
            .set("fill", scriber.fill)
            .set("stroke", scriber.stroke)
            .set("stroke-width", scriber.stroke_width)
            .set("d", data);
        viewbox.add(path)
    }
}

pub struct CircleScriber {
    z_scale_factor: f32,
}

impl CircleScriber {
    pub fn new(z_scale_factor: f32) -> Self {
        CircleScriber { z_scale_factor }
    }
}

impl HoloPointStrategy for CircleScriber {
    /// This is a scriber that uses `Element`s (namely `Circle`s) and adds those directly to the `SVG` viewbox.
    /// It draws circles around each point, scaled by the z value. (Larger z = farther away = larger circle with flatter arc)
    fn scribe_points(&self, mut viewbox: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG {
        for point in points {
            let (x, y, z) = (point.x, point.y, point.z);
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", z * self.z_scale_factor as f32)
                .set("stroke-width", scriber.stroke_width)
                .set("stroke", scriber.stroke)
                .set("fill", scriber.fill);

            viewbox = viewbox.add(circle);
        }
        viewbox
    }
}

/// Different strategies to visualize a point
pub trait HoloPointStrategy {
    fn scribe_points(&self, viewbox: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG;
}

pub struct Scriber {
    stroke: &'static str,
    stroke_width: Num,
    fill: &'static str,
    point_scribing_strategy: Box<dyn HoloPointStrategy>,
    canvas_size: (usize, usize),
    margin_percentage: Num,
}


impl Scriber {
    pub fn new(
        point_scribing_strategy: impl HoloPointStrategy + 'static,
        canvas_size: (usize, usize),
    ) -> Self {
        Self {
            stroke: "black",
            stroke_width: 0.005,
            fill: "none",
            point_scribing_strategy: Box::new(point_scribing_strategy),
            canvas_size: canvas_size,
            margin_percentage: 0.25,
        }
    }
    /*
    Assumtions:
    - x points right
    - y points up
    - z is positive out of the screen
    */
    pub fn scribe(&self, points: &Vec<Vec3>) -> svg::Document {
        // Set the size of the containing document based on the canvas size supplied by the user
        let doc = Document::new()
            .set("width", self.canvas_size.0)
            .set("height", self.canvas_size.1);

        // Build a viewbox specified by the upper and lower bounds of the coordinates of the point set, plus
        // some margin
        let (x_min, y_min, width, height) = self.find_extent(points);
        let mut viewbox = SVG::new().set("viewBox", (x_min,y_min, width, height));

        // Scribe the points into the viewbox we just made
        viewbox = self.point_scribing_strategy
            .scribe_points(viewbox, points, &self);
        doc.add(viewbox)
    }

    /// Returns (min_x, min_y, width, height) of the point set.
    /// The values are adjusted using a percentage of the raw width and height as determined by the margin
    fn find_extent(&self, points: &Vec<Vec3>) -> (Num, Num, Num, Num) {
        let (mut x_min, mut y_min) = (Num::MAX, Num::MAX);
        let (mut x_max, mut y_max) = (Num::MIN, Num::MIN);

        for point in points {
            x_min = x_min.min(point.x);
            x_max = x_max.max(point.x);
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
        }

        // The extact width and height: distance between the largest and smallest points
        let (raw_width, raw_height) = (x_max - x_min, y_max - y_min);

        // The margins we want to extend the max and min values by - calculated as a percentage of the raw width and height
        let (width_margin, height_margin) = (self.margin_percentage * raw_width, self.margin_percentage * raw_height);

        // Extend the min and max values
        x_min -= width_margin;
        y_min -= height_margin;
        x_max += width_margin;
        y_max += height_margin;

        // Return the min values and the new distances for width and height
        (x_min, y_min, (x_max - x_min), (y_max - y_min))
    }
}
