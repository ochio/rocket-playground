use dotenvy::dotenv;
use std::env;

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> String {
    let token = env::var("TOKEN").expect("FOO not set");
    let url = "https://www.rust-lang.org";
    let contents = reqwest::get(url).await.unwrap().text().await;
    format!("{}", contents.unwrap())
}

#[launch]
fn rocket() -> _ {
    dotenv().expect(".env file not found");
    rocket::build().mount("/", routes![index])
}
