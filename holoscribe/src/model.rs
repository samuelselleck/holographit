use glam::Vec3;
use obj::Obj;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::error::Error;
use std::f32::INFINITY;

pub struct ObjInterpolator {
    model: Obj,
}

impl ObjInterpolator {
    pub fn new(obj: Obj) -> Self {
        Self { model: obj }
    }

    // load an obj struct from a file
    pub fn from_file(file_path: String) -> Result<Self, Box<dyn Error>> {
        let user_defined_model: Obj = Obj::load(file_path)?;
        Ok(Self::new(user_defined_model))
    }

    // given an Obj, interpolate points between connected vertices to simulate lines
    pub fn interpolate_edges(&self, points_per_unit: usize) -> Vec<Vec3> {
        // extract vertices
        //let verts = obj.data.position.clone();
        // extract paths
        let mut poly_paths: Vec<Vec<usize>> = Vec::new();
        for simple_poly in self.model.data.objects[0].groups[0].polys.clone() {
            let mut poly_path: Vec<usize> = Vec::new();
            for vert in simple_poly.0 {
                poly_path.push(vert.0)
            }
            poly_paths.push(poly_path)
        }

        let mut edge_set = HashSet::new();
        // sort vertex indices before insert to prevent two-way paths, e.g. 5 -> 7 && 7 -> 5
        for path in poly_paths {
            edge_set.insert((min(path[0], path[1]), max(path[0], path[1])));
            edge_set.insert((min(path[1], path[2]), max(path[1], path[2])));
            edge_set.insert((min(path[2], path[0]), max(path[2], path[0])));
        }

        let mut vertex_data: Vec<Vec3> = Vec::new();
        for vertex_pair in edge_set {
            // match vertex indices to associated x,y,z coordinates
            // vertex indices are 1-indexed and array indices are 0-indexed
            let start_vert_index = vertex_pair.0;
            let end_vert_index = vertex_pair.1;
            let start_vert_position = Vec3 {
                x: self.model.data.position[start_vert_index][0],
                y: self.model.data.position[start_vert_index][1],
                z: self.model.data.position[start_vert_index][2],
            };
            let end_vert_position = Vec3 {
                x: self.model.data.position[end_vert_index][0],
                y: self.model.data.position[end_vert_index][1],
                z: self.model.data.position[end_vert_index][2],
            };

            let distance = start_vert_position.distance(end_vert_position);

            // generate a point for each point_per_unit, plus 1 point for each end of the segment
            let num_points: i32 = max(3, (distance * points_per_unit as f32) as i32);

            // insert interpolated vertices
            for i in 0..=num_points {
                let mut lerp_factor = (1.0 / num_points as f32) * i as f32;
                if lerp_factor == INFINITY {
                    lerp_factor = 0.0;
                }
                let interpolated_vertex = start_vert_position.lerp(end_vert_position, lerp_factor);
                vertex_data.push(interpolated_vertex);
            }
        }

        return vertex_data;
    }

    #[allow(dead_code)]
    pub fn generate_csv(vertices: Vec<Vec3>) {
        // everything below is for manual testing by exporting a CSV file from vertex data:
        let mut wtr = csv::Writer::from_path("./out.csv").unwrap();
        for vert in &vertices {
            wtr.write_record(&[vert.x.to_string(), vert.y.to_string(), vert.z.to_string()])
                .unwrap();
        }
        wtr.flush().unwrap();
    }
}
