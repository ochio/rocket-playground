use chrono::{Local, Timelike};
use dotenvy::dotenv;
use reqwest::Client;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tokio::time::{sleep, sleep_until, Duration, Instant};

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

// request
#[derive(Deserialize, Debug)]
struct LineEvent {
    replyToken: String,
    message: LineMessage,
}

#[derive(Deserialize, Debug)]
struct LineMessage {
    #[serde(rename = "type")]
    msg_type: String,
    text: String,
}

#[derive(Deserialize, Debug)]
struct LineWebhookRequest {
    events: Vec<LineEvent>,
}

#[derive(Serialize, Debug)]
struct LineReplyMessage {
    #[serde(rename = "type")]
    msg_type: String,
    text: String,
}

#[derive(Serialize, Debug)]
struct LineReplyBody {
    replyToken: String,
    messages: Vec<LineReplyMessage>,
}

#[post("/", format = "json", data = "<body>")]
async fn index(body: Json<LineWebhookRequest>) {
    let access_token =
        env::var("LINE_CHANNEL_ACCESS_TOKEN").expect("LINE_CHANNEL_ACCESS_TOKEN not set");
    let token = env::var("GITHUB_TOKEN").expect("Token not set");
    let user = env::var("GITHUB_USER").expect("Token not set");
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
    let query_body = json!({
        "query": query,
        "variables": variables
    })
    .to_string();
    let client = Client::new();

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "test")
        .body(query_body)
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

    for event in &body.events {
        let reply_token = &event.replyToken;

        let message = if found { "done" } else { "yet" }.to_string();

        // 応答メッセージの作成
        let reply_message = LineReplyMessage {
            msg_type: "text".to_string(),
            text: message,
        };

        let reply_body = LineReplyBody {
            replyToken: reply_token.clone(),
            messages: vec![reply_message],
        };

        // LINE APIに返信を送信
        let _response = client
            .post("https://api.line.me/v2/bot/message/reply")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&reply_body)
            .send()
            .await;
        // エラーハンドリングを追加する場合は、ここでレスポンスを確認します
    }
}

async fn send_message() {
    let token = env::var("LINE_CHANNEL_ACCESS_TOKEN").expect("LINE_CHANNEL_ACCESS_TOKEN not set");
    let user_id = env::var("USER_ID").expect("USER_ID not set");
    let client = Client::new();

    // 送信するメッセージを設定
    let reply_body = json!({
        "to": user_id,  // 送信先のユーザーIDまたはグループID
        "messages": [{
            "type": "text",
            "text": "毎日22時のメッセージです！"
        }]
    });

    let _response = client
        .post("https://api.line.me/v2/bot/message/push")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&reply_body)
        .send()
        .await;

    // エラーハンドリングを追加する場合は、ここでレスポンスを確認します
}

// #[launch]
#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let mut n = 0;
    loop {
        if n == 3 {
            return;
        }
        n = n + 1;
        let now = Local::now();
        // 次の22時までの秒数を計算
        let next_run = if now.hour() < 22 {
            // 今日の22時までの残り時間
            now.with_hour(22)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .timestamp()
                - now.timestamp()
        } else {
            // 明日の22時までの残り時間
            now.with_hour(22)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .timestamp()
                - now.timestamp()
                + 86400 // +24時間(秒数)
        };

        // 次の22時まで待機
        sleep(Duration::from_secs(next_run as u64)).await;

        send_message().await;
    }
    // rocket::build().mount("/", routes![index])
}
