use glam::Vec3;
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path, SVG};
use svg::Document;

use crate::cli::CanvasSize;

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
    fn scribe_points(&self, doc: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG {
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
        doc.add(path)
    }
}

pub struct CircleScriber {}

impl HoloPointStrategy for CircleScriber {
    /// This is a scriber that uses `Element`s (namely `Circle`s) and adds those directly to the `SVG` document.
    /// It draws circles around each point, scaled by the z value. (Larger z = farther away = larger circle with flatter arc)
    fn scribe_points(&self, mut doc: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG {
        for point in points {
            let (x, y, z) = (point.x, point.y, point.z);

            // We scale the points by half the canvas size
            let circle = Circle::new()
                .set("cx", x * scriber.canvas_size.width as f32 * 0.5)
                .set("cy", y * scriber.canvas_size.height as f32 * 0.5)
                .set("r", z * scriber.canvas_size.width as f32 * 0.5)
                .set("stroke-width", scriber.stroke_width)
                .set("stroke", scriber.stroke)
                .set("fill", scriber.fill);

            doc = doc.add(circle);
        }
        doc
    }
}

/// Different strategies to visualize a point
pub trait HoloPointStrategy {
    fn scribe_points(&self, doc: SVG, points: &Vec<Vec3>, scriber: &Scriber) -> SVG;
}

pub struct Scriber {
    stroke: &'static str,
    stroke_width: Num,
    fill: &'static str,
    point_scribing_strategy: Box<dyn HoloPointStrategy>,
    canvas_size: CanvasSize,
    margin: Num,
}


impl Scriber {
    pub fn new(
        point_scribing_strategy: impl HoloPointStrategy + 'static,
        canvas_size: CanvasSize,
    ) -> Self {
        Self {
            stroke: "black",
            stroke_width: 0.5,
            fill: "none",
            point_scribing_strategy: Box::new(point_scribing_strategy),
            canvas_size: canvas_size,
            margin: 1.0,
        }
    }
    /*
    Assumtions:
    x, y, and z are between 0 and 1
    - x points right
    - y points up
    - z is positive out of the screen
    */
    pub fn scribe(&self, points: &Vec<Vec3>) -> svg::Document {
        let ((x_min, x_max), (y_min, y_max), _) = self.bounds(points);
        let data = Data::new() //add bounding box for now
            .move_to((x_min, y_min))
            .line_to((x_min, y_max))
            .line_to((x_max, y_max))
            .line_to((x_max, y_min))
            .line_to((x_min, y_min));
        let path = Path::new()
            .set("fill", self.fill)
            .set("stroke", "red")
            .set("stroke-width", self.stroke_width)
            .set("d", data);

        // Draw the bounding box
        let doc = Document::new()
            .set("viewBox", (x_min, y_min, x_max - x_min, y_max - y_min))
            .add(path);

        // Scribe the points
        self.point_scribing_strategy
            .scribe_points(doc, points, &self)
    }

    //x, y and z point bounds + margins
    fn bounds(&self, points: &Vec<Vec3>) -> ((Num, Num), (Num, Num), (Num, Num)) {
        let (mut x_min, mut y_min, mut z_min) = (Num::MAX, Num::MAX, Num::MAX);
        let (mut x_max, mut y_max, mut z_max) = (Num::MIN, Num::MIN, Num::MIN);

        for point in points {
            x_min = x_min.min(point.x);
            x_max = x_max.max(point.x);
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
            z_min = z_min.min(point.z);
            z_max = z_max.max(point.z);
        }

        let m = self.canvas_size.width as f32 + self.margin;
        (
            (x_min - m, x_max + m),
            (y_min - m, y_max + m),
            (z_min - m, z_max + m),
        )
    }
}
