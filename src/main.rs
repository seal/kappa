#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;
use serde::Deserialize;
#[get("/<name>/<age>")]
fn wave(name: Option<String>, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[derive(Deserialize)]
struct Task {
    description: String,
    complete: bool,
}

#[post("/todo", data = "<task>")]
fn new(task: Json<Task>) { /* .. */
}
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![wave])
        .mount("/json", routes![new])
}
