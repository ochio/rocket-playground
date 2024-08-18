use chrono::Local;
use dotenvy::dotenv;
use reqwest::Client;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[macro_use]
extern crate rocket;

// Define the structures to parse the JSON response
#[derive(Deserialize, Debug)]
struct ContributionDay {
    contributionCount: u32,
    date: String,
}

#[derive(Deserialize, Debug)]
struct Week {
    contributionDays: Vec<ContributionDay>,
}

#[derive(Deserialize, Debug)]
struct ContributionCalendar {
    weeks: Vec<Week>,
}

#[derive(Deserialize, Debug)]
struct ContributionsCollection {
    contributionCalendar: ContributionCalendar,
}

#[derive(Deserialize, Debug)]
struct User {
    contributionsCollection: ContributionsCollection,
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    user: User,
}

#[derive(Deserialize, Debug)]
struct GraphQLResponse {
    data: ResponseData,
}

#[derive(Serialize, Deserialize)]
struct APIResult {
    commited: bool,
}

#[get("/")]
async fn index() -> Json<APIResult> {
    let token = env::var("GITHUB_TOKEN").expect("FOO not set");
    let user = env::var("GITHUB_USER").expect("FOO not set");
    let url = "https://api.github.com/graphql";
    let query = r#"
    query($userName:String!) {
        user(login: $userName){
          contributionsCollection {
            contributionCalendar {
              totalContributions
              weeks {
                contributionDays {
                  contributionCount
                  date
                }
              }
            }
          }
        }
      }"#;
    let variables = json!({
        "userName": user
    });
    let body = json!({
        "query": query,
        "variables": variables
    })
    .to_string();
    let client = Client::new();

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "test")
        .body(body)
        .send()
        .await
        .unwrap();

    let response_text = match response.text().await {
        Ok(text) => text,
        Err(err) => format!("Error reading response: {}", err),
    };

    let graphql_response: GraphQLResponse =
        serde_json::from_str(&response_text).unwrap_or_else(|_| panic!("Failed to parse JSON"));

    let today = Local::now().format("%Y-%m-%d").to_string(); // Replace with the date you want to check
    let mut found = false;

    for week in graphql_response
        .data
        .user
        .contributionsCollection
        .contributionCalendar
        .weeks
    {
        for day in week.contributionDays {
            if day.date == today {
                if day.contributionCount > 0 {
                    found = true;
                }
                break;
            }
        }
    }

    let result = APIResult { commited: found };

    Json(result)
}

#[launch]
fn rocket() -> _ {
    dotenv().expect(".env file not found");
    rocket::build().mount("/", routes![index])
}
