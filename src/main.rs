#[macro_use] extern crate rocket;

mod db;
mod models;
mod handlers;

use rocket_dyn_templates::Template;
use dotenvy::dotenv;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // ПРИНУДИТЕЛЬНЫЙ ВЫВОД В STDERR ДЛЯ ДИАГНОСТИКИ
    eprintln!("🚀 Starting Rocket application...");
    eprintln!("🔍 Current working directory: {:?}", std::env::current_dir().unwrap_or_else(|_| "unknown".into()));
    eprintln!("📂 DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or_else(|_| "not set".into()));

    let _ = dotenv();

    let pool = db::init_db_pool()
        .await
        .expect("❌ Failed to initialize database pool");

    db::run_migrations(&pool)
        .await
        .expect("❌ Failed to run migrations");

    eprintln!("✅ Database initialized successfully");

    let rocket = rocket::build()
        .manage(pool)
        .mount("/", routes![
            handlers::index,
            handlers::delivery_detail,
            handlers::list_deliveries,
            handlers::create_delivery,
            handlers::get_delivery,
            handlers::update_status,
            handlers::delete_delivery,
            handlers::get_history,
        ])
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}