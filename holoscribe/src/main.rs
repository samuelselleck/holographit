//responsible for transforming point clouds/obj files to a holographic svg
mod scriber;
//cli accepts obj file and svg location (and optional parameters)
mod cli;

use clap::Parser;
use cli::Args;
use glam::f32::Vec3;
use obj::Obj;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::error::Error;
use std::f32::INFINITY;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let user_defined_model = obj_from_file(args.input).unwrap();
    let interpolated_points = interpolate_edges(user_defined_model, 15);

    let circle_strat = scriber::CircleScriber {};
    let scriber = scriber::Scriber::new(circle_strat, args.canvas_size);

    let svg = scriber.scribe(&interpolated_points);

    svg::save(args.output, &svg).expect("failed to save");
    Ok(())
}

// load an obj struct from a file
fn obj_from_file(file_path: String) -> Result<Obj, Box<dyn Error>> {
    let user_defined_model: Obj = Obj::load(file_path)?;
    return Ok(user_defined_model);
}

// given an Obj, interpolate points between connected vertices to simulate lines
fn interpolate_edges(obj: Obj, points_per_unit: i32) -> Vec<Vec3> {
    // extract vertices
    //let verts = obj.data.position.clone();
    // extract paths
    let mut poly_paths: Vec<Vec<usize>> = Vec::new();
    for simple_poly in obj.data.objects[0].groups[0].polys.clone() {
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
            x: obj.data.position[start_vert_index][0],
            y: obj.data.position[start_vert_index][1],
            z: obj.data.position[start_vert_index][2],
        };
        let end_vert_position = Vec3 {
            x: obj.data.position[end_vert_index][0],
            y: obj.data.position[end_vert_index][1],
            z: obj.data.position[end_vert_index][2],
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
fn generate_csv(vertices: Vec<Vec3>) {
    // everything below is for manual testing by exporting a CSV file from vertex data:
    let mut wtr = csv::Writer::from_path("./out.csv").unwrap();
    for vert in &vertices {
        wtr.write_record(&[vert.x.to_string(), vert.y.to_string(), vert.z.to_string()])
            .unwrap();
    }
    wtr.flush().unwrap();
}

#[cfg(test)]
mod tests {}
