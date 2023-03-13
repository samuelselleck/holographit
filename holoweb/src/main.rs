use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};

use actix_files as fs;
use holoscribe::{model::ObjInterpolator, scriber};
use std::path::PathBuf;
#[actix_web::main]
async fn main() {
    println!("HOLO, world!");

    let server = HttpServer::new(|| App::new().service(get_index).service(get_static));
    // server.route("/", web::get().to(get_index));
    server
        .bind("127.0.0.1:5000")
        .expect("Error binding server!")
        .run()
        .await
        .unwrap();
    // .expect("Error running server!");
}

#[get("/temp/{filename:.*}")]
async fn get_static(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    println!("Called get_static!");
    println!("{:?}", req);
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
    println!("{:?}", file);
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[get("/")]
async fn get_index() -> HttpResponse {
    let model_path = PathBuf::from("../holoscribe/tests/cube.obj");
    let model = ObjInterpolator::from_file(model_path.to_str().unwrap().to_string())
        .expect("Valid OBJ model");
    let points_per_unit: usize = 25;
    let interpolated_points = model.interpolate_edges(points_per_unit);
    let circle_strat = scriber::CircleScriber::new();
    let canvas_size = (500, 500);
    let scriber = scriber::Scriber::new(circle_strat, canvas_size);
    let svg = scriber.scribe(&interpolated_points);
    svg::save("temp/cube.svg", &svg).expect("Error saving SVG");
    let resp = HttpResponse::Ok().content_type("text/html").body(
        r#"
        <html><head>
        <title>HOLO World!</title></head><body>
        <img src="temp/cube.svg">
        Hey what's up?
        </body></html>
        "#,
    );
    resp
}
