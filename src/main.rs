#[macro_use]
extern crate rocket;

use log::info;
use rocket::{Build, Rocket};

use register::catchers;
use register::config::{init, Env};
use register::db;
use register::routes;

#[launch]
fn rocket() -> Rocket<Build> {
    // 1. Init logger and env
    let env: Env = init();

    // 2. Init Rocket
    // a) connect to DB
    // b) define APIs
    // c) define error handlers
    info!(target: "app", "Starting Rocket...");
    rocket::build()
        .attach(db::init(env))
        .mount(
            "/",
            routes![
                routes::api::post_register_temperature,
                routes::api::post_register_humidity,
                routes::api::post_register_light,
                routes::api::post_register_motion,
                routes::api::post_register_airquality,
                routes::api::post_register_airpressure,
                routes::api::get_sensor_value,
                routes::api::keep_alive,
            ],
        )
        .register(
            "/",
            catchers![
                catchers::bad_request,
                catchers::not_found,
                catchers::internal_server_error,
            ],
        )
}

// testing
#[cfg(test)]
mod tests_integration;
