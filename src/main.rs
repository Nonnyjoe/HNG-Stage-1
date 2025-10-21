mod config;
mod routes;
use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, middleware::Logger, web};
use config::{config::Config, config_scope};
use dotenv::dotenv;

pub struct AppState {
    pub env: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Loading environment variables from .env file.....");
    dotenv().ok();
    env_logger::init();

    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "actix_web=info");
        }
    }
    println!("Starting Server......");
    let config = Config::init();

    let port: u16 = config.port.parse().expect("PORT must be i16 type");
    let url: String = config.url.clone().parse().expect("URL must be String type");

    let app_state = web::Data::new(AppState { env: config });

    println!("Server Started and running on {}:{}......", url, port);
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .configure(config_scope::config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind((url, port))?
    .run()
    .await
}
