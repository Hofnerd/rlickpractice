use std::fs;

use askama_axum::Template;
use dotenv::dotenv;
use anyhow::Context;
use axum::{
    extract::Multipart,
    extract::State, 
    http::StatusCode, 
    response::IntoResponse, 
    routing::get, 
    Router
};
use serde::Deserialize;
use sqlx::MySqlPool;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response();
    }
}

impl<E> From<E> for AppError
    where
        E: Into<anyhow::Error>, {
    fn from(err: E) -> Self {
        return Self(err.into());
    }
}

#[derive(Deserialize, Debug, Template)]
#[template(path = "licks.html")]
pub struct LicksDisplay {
    pub licks: Vec<Lick>,
}

impl LicksDisplay{
    fn new(licks: Vec<Lick>) -> Self {
        let ld = LicksDisplay {
            licks
        };
        return ld;
    }
}


#[derive(Deserialize, Debug, Template)]
#[template(path = "lick_pdf.html")]
pub struct PdfDisplay {
    pub pdf: String,
}

impl PdfDisplay{
    fn new(filename: String) -> Self {
        let ld = PdfDisplay {
            pdf: filename
        };
        return ld;
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Deserialize)]
pub struct DbQuery {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct DbAdd {
    pub file: String,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct Lick {
    id : i32,
    filename : String,
    learned : i32,
}

async fn index() -> IndexTemplate{
    return IndexTemplate;
}

async fn get_licks(pool: &MySqlPool) -> Result<Vec<Lick>, anyhow::Error> {
    let tmp: String = format!("SELECT id,filename,learned FROM Licks");
    let licks = sqlx::query_as(&tmp).fetch_all(pool).await?;
    Ok(licks)
}

async fn list_licks(State(pool): State<MySqlPool>) -> Result<LicksDisplay, AppError> {
    let licks = get_licks(&pool).await?;
    return Ok(LicksDisplay::new(licks));
}

async fn grab_file(pool: &MySqlPool) -> Result<String, anyhow::Error> {
    let tmp: String = format!("SELECT id,filename,learned from Licks LIMIT 1");
    let lick: Lick = sqlx::query_as(&tmp).fetch_one(pool).await?;
    return Ok(lick.filename);
}

async fn serve_pdf(State(pool): State<MySqlPool>) -> Result<PdfDisplay, AppError> {
    let filename = grab_file(&pool).await?;
    return Ok(PdfDisplay::new(filename));
}

async fn add_lick_db(pool: &MySqlPool, filename: &String) -> Result<i32, anyhow::Error> {
    let tmp = format!("INSERT INTO Licks (filename) values ('{}')", filename);
    sqlx::query(&tmp).execute(pool).await?;
    return Ok(0);
}

async fn upload_lick_pdf(State(pool):State<MySqlPool>, mut multipart: Multipart) -> IndexTemplate{
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let ctype = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        println!("Length of '{}' is {} bytes; type {}", 
            name, 
            data.len(),
            ctype);
        if ctype.contains("pdf") {
            let filename = format!("./data/{}.pdf",name);
            let _ = fs::write(&filename, data);
            let _ = add_lick_db(&pool, &filename).await;
        }
    }

    return IndexTemplate;
}

#[tokio::main]
async fn main() ->anyhow::Result<()>{
    // Use dotenv to load the .env file for the database url
    dotenv().ok();
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = sqlx::MySqlPool::connect(&db_connection_str)
        .await
        .context("cant connect to database")?;

    // Build our app with a single orute
    let app = Router::new()
        .route("/", get(index).post(upload_lick_pdf))
        .route("/licks", get(list_licks))
        .route("/pdf", get(serve_pdf))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
} 
