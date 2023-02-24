//responsible for transforming point clouds/obj files to a holographic svg
mod scriber;
//cli accepts obj file and svg location (and optional parameters)
mod cli;

use obj::Obj;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: use Clap if this gets unwieldy in the future
    let args: Vec<String> = env::args().collect();
    let input_model_file_path = args[1].clone();

    let user_defined_model = obj_from_file(input_model_file_path).unwrap();
    let verts = user_defined_model.data.position;
    dbg!(verts);

    Ok(())
}

// load an obj struct from a file
fn obj_from_file(file_path: String) -> Result<Obj, Box<dyn Error>> {
    let user_defined_model: Obj = Obj::load(file_path)?;
    return Ok(user_defined_model);
}

#[cfg(test)]
mod tests {
    use crate::obj_from_file;
}
