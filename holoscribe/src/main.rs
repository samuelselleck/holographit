//responsible for transforming point clouds/obj files to a holographic svg
mod scriber;
//cli accepts obj file and svg location (and optional parameters)

mod cli;
mod model;

use std::error::Error;

use crate::model::ObjInterpolator;
use clap::Parser;
use cli::Args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let user_defined_model = ObjInterpolator::from_file(args.input).unwrap();
    let interpolated_points = user_defined_model.interpolate_edges(args.stroke_density);

    let circle_strat = scriber::CircleScriber {};
    let scriber = scriber::Scriber::new(circle_strat, args.canvas_size);

    let svg = scriber.scribe(&interpolated_points);

    svg::save(args.output, &svg).expect("failed to save");
    Ok(())
}

#[cfg(test)]
mod tests {}
