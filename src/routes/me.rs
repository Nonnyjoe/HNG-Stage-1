// use crate::AppState;
// use actix_web::{HttpResponse, Responder, get, web};
// use rand::Rng;
// use reqwest::Client;


// #[get("/me")]
// async fn me(_data: web::Data<AppState>) -> impl Responder {
//     let page = rand::rng().random_range(1..=34);
//     let cat_url = &_data.env.cat_fact_source_url;

//     println!("Cat Fact Source URL: {}", cat_url);
//     println!("Random Page Number: {}", page);

//     // Fetch cat facts from the API
//     let client = Client::new();
//     let url = format!("{}?page={}", cat_url, page);
//     let mut fact: Option<String> = None;
//     let mut error: String = String::new();


//     match client.get(&url).send().await {
//         Ok(resp) => match resp.json::<serde_json::Value>().await {
//             Ok(json) => {
//                 let facts_array = json.get("data").and_then(|d| d.as_array()).cloned();
//                 if let Some(facts_array) = facts_array {
//                     if !facts_array.is_empty() {
//                         let idx = rand::rng().random_range(0..facts_array.len());
//                         let user_fact = facts_array[idx].get("fact").and_then(|f| f.as_str()).unwrap_or("").to_string();
//                         fact = Some(user_fact.clone());
//                         println!("Random Cat Fact: {}", user_fact);
//                     } else {
//                         println!("No facts found in the response: /n{:?}", json);
//                         error = format!("No facts found in the response, because: {:?}", json).to_string();
//                     }
//                 } else {
//                     println!("No facts found in the response: /n{:?}", json);
//                     error = format!("No facts found in the response, because: {:?}", json).to_string();
//                 }
//             }
//             Err(_) =>{ println!("Failed to parse JSON response"); fact = None; error = "Failed to parse JSON response".to_string()}
//         },
//         Err(e) =>{ println!("Failed to fetch cat facts {:?}", e); fact = None; error = "Failed to fetch cat facts".to_string()}
//     };

//     if !fact.is_none() {
//         let json_response = serde_json::json!({
//             "status": "success",
//             "user": serde_json::json!({
//                 "name": _data.env.name,
//                 "email": _data.env.email,
//                 "stack": "Rust, Actix-web, PostgreSQL"
//             }),
//             "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
//             "fact": fact.unwrap_or_default()
//         });
//         return HttpResponse::Ok().json(json_response)
//     } else {
//         let json_response = serde_json::json!({
//             "status": "error",
//             "message": format!("{}. Please try again later.", error),
//             "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
//             "user": serde_json::json!({
//                 "name": _data.env.name,
//                 "email": _data.env.email,
//                 "stack": "Rust, Actix-web, PostgreSQL"
//             }),
//         });
//         return HttpResponse::InternalServerError().json(json_response)
//     }

// }