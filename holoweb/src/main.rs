use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;
use tera::{self, Context, Tera};

use actix_files as fs;
use holoscribe::{model::ObjInterpolator, scriber};
use holoviz::Visualizer;
use std::fs as stdfs;
use std::path::PathBuf;
use std::str;

#[derive(Debug, Deserialize)]
struct VisParameters {
    model_file: PathBuf,
    width_mm: usize,
    height_mm: usize,
    stroke_density: usize,
    light_source_start_x: f32,
    light_source_start_y: f32,
    light_source_end_x: f32,
    light_source_end_y: f32,
    duration_s: f32,
}

// impl FromRequest for ScribeParameters {
//     fn from
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("HOLO, world!");

    let server = HttpServer::new(|| {
        App::new() // TODO: Better way to structure this?
            .service(get_index)
            .service(get_static)
            // .route("/scriber", web::get().to(get_scriber))
            .service(get_visualizer)
    });
    // server.route("/", web::get().to(get_index));
    server
        .bind("127.0.0.1:5000")?
        // .expect("Error binding server!")
        .run()
        .await
    // .unwrap(); // TODO: Make main return a Result
}

#[get("/temp/{filename:.*}")]
async fn get_static(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: PathBuf = ["temp", req.match_info().query("filename")]
        .iter()
        .collect();
    // println!("Looking for: {:?}", path);
    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    // println!("{:?}", file);
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[post("/visualizer")]
async fn get_visualizer(form: web::Form<VisParameters>) -> impl Responder {
    // TODO: Input validation! (Is web form validation enough?)

    // println!("Got {:?}", form);
    // let model_path = PathBuf::from("static/dodecahedron.obj");

    let model = ObjInterpolator::from_file(form.model_file.to_str().unwrap().to_string())
        .expect("Valid OBJ model");
    let interpolated_points = model.interpolate_edges(form.stroke_density);
    let circle_strat = scriber::CircleScriber::new();
    let canvas_size = (form.width_mm, form.height_mm);
    let scriber = scriber::Scriber::new(circle_strat, canvas_size);
    let svg = scriber.scribe(&interpolated_points);
    // TODO: Don't save to a temporary file. Requires more fixes for the
    // visualizer library such that a visualizer can be built from a
    // svg struct.
    svg::save("temp/scriber.svg", &svg).expect("Error saving scriber SVG");

    // TODO: Skip writing to a buffer and just pass along the
    // SVG document instead.
    // let buf = Vec::<u8>::new();
    // svg::write(buf.clone(), &svg).expect("Error writing SVG");
    // println!("Buffer Contents: {:?}", buf);
    // let contents = str::from_utf8(&buf).unwrap();
    // println!("SVG Contents: {:?}", contents);

    let viz = Visualizer::from_file(PathBuf::from("temp/scriber.svg"))
        .expect("Error building visualizer");

    let ls_start = holoviz::Point {
        x: form.light_source_start_x,
        y: form.light_source_start_y,
    };
    let ls_end = holoviz::Point {
        x: form.light_source_end_x,
        y: form.light_source_end_y,
    };
    let hologram = viz.build_animated_hologram(&ls_start, &ls_end, form.duration_s);

    // TODO: Rather than save a temporary file on the server, just serve
    // the SVG code directly on the webpage
    svg::save("temp/visualizer.svg", &hologram).expect("Error saving SVG");
    let response = format!(
        r#"
    <html><head><title>Visualizer!</title></head>
    <body>
    <p>Scribing {} x {} image at density of {}</p>
    <img src="temp/visualizer.svg">
    "#,
        &form.width_mm, &form.height_mm, &form.stroke_density
    );
    HttpResponse::Ok().content_type("text/html").body(response)
}

#[get("/")]
async fn get_index() -> HttpResponse {
    // TODO: Have this pull in HTML code from outside of the code
    let index = render_template().expect("Error rendering template");
    let resp = HttpResponse::Ok().content_type("text/html").body(index);
    resp
}
fn render_template() -> Result<String, tera::Error> {
    let obj_dir = PathBuf::from("static");
    let tera = match Tera::new("templates/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Terra Erra: {}", e);
            ::std::process::exit(1);
        }
    };
    let mut context = Context::new();
    if let Some(model_list) = list_obj_files(obj_dir).ok() {
        context.insert("model_list", &model_list);
    }
    let s = tera.render("index.html", &context)?;
    Ok(s)
}

fn list_obj_files(directory: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::<PathBuf>::new();
    for file in stdfs::read_dir(directory)? {
        if let Some(entry) = file.ok() {
            // println!("found {:?}", entry.path());
            files.push(entry.path());
        }
    }
    Ok(files)
}
