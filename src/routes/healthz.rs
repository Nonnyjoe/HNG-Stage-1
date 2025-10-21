use crate::AppState;
use actix_web::{HttpResponse, Responder, get, web};


#[get("/healthz")]
async fn check_health(_data: web::Data<AppState>) -> impl Responder {
    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "health": "Server is active"
        })
    });

    HttpResponse::Ok().json(json_response)
}
