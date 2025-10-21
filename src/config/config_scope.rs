use crate::routes::healthz::check_health;
// use crate::routes::me::me;
use crate::routes::strings::{process_string, get_string_details, delete_string, get_strings_filtered, filter_by_natural_language};
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/v1").service(check_health).service(process_string).service(filter_by_natural_language).service(get_strings_filtered).service(get_string_details).service(delete_string);
    conf.service(scope);
}
