use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, get_service, post},
    Json, Router,
};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;
use tower_http::services::ServeDir;

use crate::models::{NewStudent, Student};
use crate::port::{AttendancePort, FingerprintPort};

/// Create a router with the fingerprint port layered on top
pub async fn create_router(port: FingerprintPort) -> Router {
    let db_pool = create_db_pool().await;

    Router::new()
        .route("/get-students", get(get_students))
        .route("/add-student", post(add_student))
        .route("/take-attendance", get(take_attendance))
        .route("/reset-attendance", post(reset_attendance))
        .route("/clear-fingerprints", post(clear_fingerprints))
        .nest_service("/", get_service(ServeDir::new("assets")))
        .layer(Extension(db_pool))
        .layer(Extension(port))
}

async fn get_students(
    Extension(db_pool): Extension<Pool<Sqlite>>,
) -> Result<Json<Vec<Student>>, (StatusCode, String)> {
    let students = sqlx::query_as!(
        Student,
        r#"
        SELECT name, student_id, fingerprint_id, attendance
        FROM student
        "#
    )
    .fetch_all(&db_pool)
    .await
    .map_err(map_db_error)?;

    Ok(Json(students))
}

async fn add_student(
    Extension(db_pool): Extension<Pool<Sqlite>>,
    Extension(port): Extension<FingerprintPort>,
    Json(new_student): Json<NewStudent>,
) -> Result<String, (StatusCode, String)> {
    println!("Adding student {}", new_student.name);

    let student_with_highest_fingerprint_id = sqlx::query_as!(
        Student,
        r#"
        SELECT name, student_id, fingerprint_id, attendance FROM student ORDER BY fingerprint_id DESC LIMIT 1;
        "#
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(map_db_error)?;

    let new_fingerprint_id = match student_with_highest_fingerprint_id {
        Some(student) => student.fingerprint_id + 1,
        None => 1,
    };

    port.enroll_fingerprint(new_fingerprint_id as u8)
        .map_err(map_port_error)?;

    sqlx::query!(
        r#"
        INSERT INTO student (name, student_id, fingerprint_id, attendance)
        VALUES (?, ?, ?, ?)
        "#,
        new_student.name,
        new_student.student_id,
        new_fingerprint_id,
        false,
    )
    .execute(&db_pool)
    .await
    .map_err(map_db_error)?;

    Ok(format!(
        "Successfully added {} with ID: {}",
        new_student.name, new_student.student_id
    )
    .to_string())
}

async fn take_attendance(
    Extension(db_pool): Extension<Pool<Sqlite>>,
    Extension(port): Extension<FingerprintPort>,
) -> Result<String, (StatusCode, String)> {
    let fingerprint_id = port.match_fingerprint().map_err(map_port_error)?;
    println!("Matched fingerprint ID: {}", fingerprint_id);

    let student = sqlx::query_as!(
        Student,
        r#"
        SELECT name, student_id, fingerprint_id, attendance
        FROM student
        WHERE fingerprint_id = ?
        "#,
        fingerprint_id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(map_db_error)?;

    let student = match student {
        Some(student) => student,
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Fingerprint was found but no student matched it".to_string(),
            ))
        }
    };

    if student.attendance {
        return Ok(format!("{} has already taken attendance", student.name));
    }

    sqlx::query!(
        r#"
        UPDATE student
        SET attendance = ?
        WHERE fingerprint_id = ?
        "#,
        true,
        fingerprint_id
    )
    .execute(&db_pool)
    .await
    .map_err(map_db_error)?;

    Ok(format!("Successfully took attendance for {}", student.name))
}

async fn reset_attendance(
    Extension(db_pool): Extension<Pool<Sqlite>>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query!(
        r#"
        UPDATE student
        SET attendance = ?
        "#,
        false
    )
    .execute(&db_pool)
    .await
    .map_err(map_db_error)?;

    Ok("Successfully reset attendance".to_string())
}

async fn clear_fingerprints(
    Extension(db_pool): Extension<Pool<Sqlite>>,
    Extension(port): Extension<FingerprintPort>,
) -> Result<String, (StatusCode, String)> {
    port.clear_fingerprints().map_err(map_port_error)?;

    sqlx::query!(
        r#"
        DELETE FROM student
        "#
    )
    .execute(&db_pool)
    .await
    .map_err(map_db_error)?;

    Ok("Successfully cleared all templates".to_string())
}

async fn create_db_pool() -> Pool<Sqlite> {
    dotenv::dotenv().expect("Unable to load the .env file. Make sure it's in the directory root.");

    let db_url = env::var("DATABASE_URL").expect("Missing `DATABASE_URL` in .env file.");

    if !Sqlite::database_exists(&db_url).await.unwrap() {
        println!("Creating database {}", &db_url);

        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("Successfully created the database"),
            Err(e) => panic!("Error creating the database: {:?}", e),
        }
    } else {
        println!("The database already exists!");
    }

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .unwrap()
}

/// Convert any database error into an HTTP status code for handlers to propagate away
fn map_db_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database error: {:?}", e);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "A database error occurred".to_string(),
    )
}

/// Convert any PortError into an HTTP status code for handlers to propagate away
fn map_port_error(e: <FingerprintPort as AttendancePort>::Error) -> (StatusCode, String) {
    eprintln!("Port error: {:?}", e);

    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
