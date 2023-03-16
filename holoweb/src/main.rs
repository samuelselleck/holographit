use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use actix_files as fs;
use holoscribe::{model::ObjInterpolator, scriber};
// use holoviz::Visualizer;
use std::path::PathBuf;
use std::str;

#[derive(Debug, Deserialize)]
struct ScribeParameters {
    width_mm: usize,
    height_mm: usize,
    stroke_density: usize,
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
            .service(get_scriber)
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

#[post("/scriber")]
async fn get_scriber(form: web::Form<ScribeParameters>) -> impl Responder {
    // TODO: Input validation!

    // println!("Got {:?}", form);
    let model_path = PathBuf::from("../holoscribe/tests/icosahedron.obj");
    let model = ObjInterpolator::from_file(model_path.to_str().unwrap().to_string())
        .expect("Valid OBJ model");
    let interpolated_points = model.interpolate_edges(form.stroke_density);
    let circle_strat = scriber::CircleScriber::new();
    let canvas_size = (form.width_mm, form.height_mm);
    let scriber = scriber::Scriber::new(circle_strat, canvas_size);
    let svg = scriber.scribe(&interpolated_points);

    // TODO: Skip writing to a buffer and just pass along the
    // SVG document instead.
    let buf = Vec::<u8>::new();
    svg::write(buf.clone(), &svg).expect("Error writing SVG");
    let contents = str::from_utf8(&buf);

    // TODO: Fix viz library so that this import will work.
    // let viz = Vizualizer::from_svg_contents(svg);

    // TODO: Rather than save a temporary file on the server, just serve
    // the SVG code directly on the webpage
    svg::save("temp/cube.svg", &svg).expect("Error saving SVG");
    let response = format!(
        r#"
    <html><head><title>Scriber!</title></head>
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
        <form action="/scriber" method="post">
        Width: <input type="text" name="width_mm"><br>
        Height: <input type="text" name="height_mm"><br>
        Stroke Density: <input type="text" name="stroke_density"><br>
        <button type="submit">Scribe!</button>
        </form>
        </body></html>
        "#,
    );
    resp
}
