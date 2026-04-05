use rocket::{Build, Rocket};
use tracing::info;

use register::catchers;
use register::config::{AppEnv, Env, init};
use register::db;
use register::routes;

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    // 1. Init logger and env
    let (env, app_env): (Env, AppEnv) = init();

    // 2. Init Rocket
    // a) connect to DB
    // b) define APIs
    // c) define error handlers
    info!(target: "app", "Starting Rocket...");
    rocket::build()
        .attach(db::init(env, app_env))
        .mount(
            "/",
            rocket::routes![routes::api::post_register, routes::api::get_sensor_value, routes::api::keep_alive,],
        )
        .register(
            "/",
            rocket::catchers![
                catchers::bad_request,
                catchers::not_found,
                catchers::internal_server_error,
                catchers::service_unavailable,
            ],
        )
}

// testing
#[cfg(test)]
mod tests_integration;
