use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use actix_files as fs;
use holoscribe::{model::ObjInterpolator, scriber};
use holoviz::Visualizer;
use std::path::PathBuf;
use std::str;

#[derive(Debug, Deserialize)]
struct VisParameters {
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
            .service(echo)
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
    // println!("Called get_static!");
    // println!("{:?}", req);
    let path: PathBuf = ["temp", req.match_info().query("filename")]
        .iter()
        .collect();
    println!("Looking for: {:?}", path);
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
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("Calling echo!");
    HttpResponse::Ok().body(req_body)
}

#[post("/visualizer")]
async fn get_visualizer(form: web::Form<VisParameters>) -> impl Responder {
    // TODO: Input validation! (Is web form validation enough?)

    // println!("Got {:?}", form);
    let model_path = PathBuf::from("../holoscribe/tests/icosahedron.obj");
    let model = ObjInterpolator::from_file(model_path.to_str().unwrap().to_string())
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
    svg::save("temp/cube.svg", &hologram).expect("Error saving SVG");
    let response = format!(
        r#"
    <html><head><title>Visualizer!</title></head>
    <body>
    <p>Scribing {} x {} image at density of {}</p>
    <img src="temp/cube.svg">
    "#,
        &form.width_mm, &form.height_mm, &form.stroke_density
    );
    HttpResponse::Ok().content_type("text/html").body(response)
}

#[get("/")]
async fn get_index() -> HttpResponse {
    // TODO: Have this pull in HTML code from outside of the code
    let resp = HttpResponse::Ok().content_type("text/html").body(
        r#"
        <html><head>
        <title>HOLO World!</title></head><body>
        <style>
        #number {
            width: 8em;
        }
        </style>
        <form action="/visualizer" method="post">
        Width: <input id="number" type="number" name="width_mm" min="100" max="2000" value="500" step="50"><br>
        Height: <input id="number" type="number" name="height_mm" min="100" max="2000" value="500" step="50"><br>
        Stroke Density: <input id="number" type="number" name="stroke_density" min="5" max="100" value="20"><br>
        Light Source Start (x, y):
        <input id="number" type="number" name="light_source_start_x" value="250" step="10">
        <input id="number" type="number" name="light_source_start_y" value="-100" step="10">
        <br>
        Light Source End (x, y):
        <input id="number" type="number" name="light_source_end_x" value="250" step="10">
        <input id="number" type="number" name="light_source_end_y" value="-100" step="10">
        <br>
        Animation Duration (s): <input id="number" type="number" name="duration_s" step="0.1" value="2.0" min="0.1" max="10.0"><br>
        <button type="submit">Visualize!</button>
        </form>
        </body></html>
        "#,
    );
    resp
}
