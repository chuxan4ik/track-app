use rocket::serde::json::Json;
use rocket::State;
use rocket::response::status::Created;
use rocket::http::Status;
use rocket_dyn_templates::Template;
use serde_json::json;
use chrono::Utc;
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{Delivery, NewDelivery, StatusUpdate, StatusHistory};


#[get("/")]
pub async fn index(pool: &State<DbPool>) -> Template {
    let deliveries: Vec<Delivery> = sqlx::query_as::<_, Delivery>(
        "SELECT id, tracking_number, description, sender, recipient, current_status, created_at, updated_at
         FROM deliveries ORDER BY created_at DESC"
    )
    .fetch_all(&**pool)
    .await
    .unwrap_or_else(|_| vec![]);
    
    Template::render("index", json!({ "deliveries": deliveries }))
}

#[get("/delivery/<id>")]
pub async fn delivery_detail(pool: &State<DbPool>, id: i32) -> Result<Template, Status> {
    let delivery_opt: Option<Delivery> = sqlx::query_as::<_, Delivery>(
        "SELECT id, tracking_number, description, sender, recipient, current_status, created_at, updated_at
         FROM deliveries WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&**pool)
    .await
    .map_err(|_| Status::InternalServerError)?;

    let delivery = match delivery_opt {
        Some(d) => d,
        None => return Err(Status::NotFound),
    };

    let history: Vec<StatusHistory> = sqlx::query_as::<_, StatusHistory>(
        "SELECT id, delivery_id, status, changed_at
         FROM status_history WHERE delivery_id = ?
         ORDER BY changed_at"
    )
    .bind(id)
    .fetch_all(&**pool)
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("delivery", json!({ "delivery": delivery, "history": history })))
}


#[get("/api/deliveries")]
pub async fn list_deliveries(pool: &State<DbPool>) -> Result<Json<Vec<Delivery>>, Status> {
    let deliveries: Vec<Delivery> = sqlx::query_as::<_, Delivery>(
        "SELECT id, tracking_number, description, sender, recipient, current_status, created_at, updated_at
         FROM deliveries ORDER BY created_at DESC"
    )
    .fetch_all(&**pool)
    .await
    .map_err(|_| Status::InternalServerError)?;
    Ok(Json(deliveries))
}

#[post("/api/deliveries", format = "json", data = "<new_delivery>")]
pub async fn create_delivery(pool: &State<DbPool>, new_delivery: Json<NewDelivery>) -> Result<Created<Json<Delivery>>, Status> {
    let tracking_number = if new_delivery.tracking_number.trim().is_empty() {
        Uuid::new_v4().to_string()
    } else {
        new_delivery.tracking_number.clone()
    };
    let now = Utc::now();
    let initial_status = "Создана".to_string();

    let mut tx = pool.begin().await.map_err(|_| Status::InternalServerError)?;

    let delivery: Delivery = sqlx::query_as::<_, Delivery>(
        "INSERT INTO deliveries (tracking_number, description, sender, recipient, current_status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         RETURNING id, tracking_number, description, sender, recipient, current_status, created_at, updated_at"
    )
    .bind(tracking_number)
    .bind(&new_delivery.description)
    .bind(&new_delivery.sender)
    .bind(&new_delivery.recipient)
    .bind(&initial_status)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    sqlx::query(
        "INSERT INTO status_history (delivery_id, status, changed_at) VALUES (?, ?, ?)"
    )
    .bind(delivery.id)
    .bind(&initial_status)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    tx.commit().await.map_err(|_| Status::InternalServerError)?;

    Ok(Created::new(format!("/api/deliveries/{}", delivery.id)).body(Json(delivery)))
}

#[get("/api/deliveries/<id>")]
pub async fn get_delivery(pool: &State<DbPool>, id: i32) -> Result<Json<Delivery>, Status> {
    let delivery_opt: Option<Delivery> = sqlx::query_as::<_, Delivery>(
        "SELECT id, tracking_number, description, sender, recipient, current_status, created_at, updated_at
         FROM deliveries WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&**pool)
    .await
    .map_err(|_| Status::InternalServerError)?;

    match delivery_opt {
        Some(d) => Ok(Json(d)),
        None => Err(Status::NotFound),
    }
}

#[put("/api/deliveries/<id>/status", format = "json", data = "<update>")]
pub async fn update_status(pool: &State<DbPool>, id: i32, update: Json<StatusUpdate>) -> Result<Json<Delivery>, Status> {
    let now = Utc::now();

    let mut tx = pool.begin().await.map_err(|_| Status::InternalServerError)?;

    let delivery_opt: Option<Delivery> = sqlx::query_as::<_, Delivery>(
        "UPDATE deliveries SET current_status = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, tracking_number, description, sender, recipient, current_status, created_at, updated_at"
    )
    .bind(&update.status)
    .bind(now)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    let delivery = match delivery_opt {
        Some(d) => d,
        None => return Err(Status::NotFound),
    };

    sqlx::query(
        "INSERT INTO status_history (delivery_id, status, changed_at) VALUES (?, ?, ?)"
    )
    .bind(id)
    .bind(&update.status)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|_| Status::InternalServerError)?;

    tx.commit().await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(delivery))
}

#[delete("/api/deliveries/<id>")]
pub async fn delete_delivery(pool: &State<DbPool>, id: i32) -> Result<Status, Status> {
    let result = sqlx::query("DELETE FROM deliveries WHERE id = ?")
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if result.rows_affected() == 0 {
        Err(Status::NotFound)
    } else {
        Ok(Status::NoContent)
    }
}

#[get("/api/deliveries/<id>/history")]
pub async fn get_history(pool: &State<DbPool>, id: i32) -> Result<Json<Vec<StatusHistory>>, Status> {
    let history: Vec<StatusHistory> = sqlx::query_as::<_, StatusHistory>(
        "SELECT id, delivery_id, status, changed_at
         FROM status_history WHERE delivery_id = ?
         ORDER BY changed_at"
    )
    .bind(id)
    .fetch_all(&**pool)
    .await
    .map_err(|_| Status::InternalServerError)?;
    Ok(Json(history))
}