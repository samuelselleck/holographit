#![feature(test)]
pub mod model;
pub mod scriber;
extern crate test;

#[cfg(test)]
mod tests {

    use super::*;
    use model::ObjInterpolator;
    use test::Bencher;

    #[bench]
    fn benchmark_interpolate_points(b: &mut Bencher) {
        let model =
            ObjInterpolator::from_file("tests/icosahedron.obj".to_string()).expect("invalid model");
        b.iter(|| model.interpolate_edges(100));
    }

    #[bench]
    fn benchmark_scribe(b: &mut Bencher) {
        let model =
            ObjInterpolator::from_file("tests/icosahedron.obj".to_string()).expect("invalid model");
        let interpolated_points = model.interpolate_edges(100);
        let circle_strat = scriber::CircleScriber::new(1.0);
        let scriber = scriber::Scriber::new(circle_strat, (100, 100));
        b.iter(|| scriber.scribe(&interpolated_points));
    }
}
