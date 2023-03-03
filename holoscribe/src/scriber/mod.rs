use glam::Vec3;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

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
    fn scribe_point(&self, data: Data, point: &Vec3) -> Data {
        let x = point.x;
        let y = point.y;
        let z = point.z;
        let z = self.map_range(z);
        data.move_to((x, y))
            .move_by((0, -z))
            .line_by((-z, z))
            .line_by((z, z))
            .line_by((z, -z))
            .line_by((-z, -z))
    }
}

//This could later on use different strategies to visualize a point
//ie. perfect, arc, something else interesting
pub trait HoloPointStrategy {
    fn scribe_point(&self, data: Data, point: &Vec3) -> Data;
}

pub struct Scriber {
    //Ammount to increase the document view in x/y beyond furthest away points
    //This effects if some arcs are drawn outisde the work area
    margin: Num,
    stroke: &'static str,
    stroke_width: Num,
    point_scribing_strategy: Box<dyn HoloPointStrategy>,
}

type Num = f32;

impl Scriber {
    pub fn new(point_scribing_strategy: impl HoloPointStrategy + 'static) -> Self {
        Self {
            margin: 1.0,
            stroke: "black",
            stroke_width: 0.05,
            point_scribing_strategy: Box::new(point_scribing_strategy),
        }
    }
    /*
    Assumtions:
    - x points right
    - y points up
    - z is positive out of the screen
    */
    pub fn scribe(&self, points: &Vec<Vec3>) -> svg::Document {
        let ((x_min, x_max), (y_min, y_max), _) = self.bounds(points);
        let data = points.iter().fold(Data::new(), |d, p| {
            self.point_scribing_strategy.scribe_point(d, p)
        });
        let data = data //add bounding box for now
            .move_to((x_min, y_min))
            .line_to((x_min, y_max))
            .line_to((x_max, y_max))
            .line_to((x_max, y_min))
            .line_to((x_min, y_min));
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", self.stroke)
            .set("stroke-width", self.stroke_width)
            .set("d", data);
        Document::new()
            .set("viewBox", (x_min, y_min, x_max - x_min, y_max - y_min))
            .add(path)
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

        let m = self.margin;
        (
            (x_min - m, x_max + m),
            (y_min - m, y_max + m),
            (z_min - m, z_max + m),
        )
    }
}
