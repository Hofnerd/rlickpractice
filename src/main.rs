use askama_axum::Template;
use dotenv::dotenv;
use anyhow::Context;
use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Form, Router
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

async fn add_lick_db(pool: &MySqlPool, filename: &String) -> Result<i32, anyhow::Error> {
    let tmp = format!("INSERT INTO Licks (filename) values ('{}')", filename);
    sqlx::query(&tmp).execute(pool).await?;
    return Ok(0);
}

async fn add_lick(State(pool): State<MySqlPool>, Form(params): Form<DbAdd>) -> IndexTemplate{
    let filename = params.file;
    if filename.eq("") {
            return IndexTemplate;
    }

    match add_lick_db(&pool, &filename).await {
        Ok(_) => {
            return IndexTemplate;
        }
        Err(_) => {
            return IndexTemplate;
        }
    }
}

async fn list_licks(State(pool): State<MySqlPool>) -> Result<LicksDisplay, AppError> {
    let licks = get_licks(&pool).await?;
    return Ok(LicksDisplay::new(licks));
}

async fn get_licks(pool: &MySqlPool) -> Result<Vec<Lick>, anyhow::Error> {
    let tmp: String = format!("SELECT id,filename,learned FROM Licks");
    let licks = sqlx::query_as(&tmp).fetch_all(pool).await?;
    Ok(licks)
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
        .route("/", get(index))
        .route("/licks", get(list_licks))
        .route("/add_lick", post(add_lick))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
} 
