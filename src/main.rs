use askama::Template;
use dotenv::dotenv;
use anyhow::{anyhow, Context};
use axum::{
    extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get, Router
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
#[template(path = "lick.html")]
pub struct LickDisplay {
    pub id : i32,
    pub filename: String,
    pub learned: i32 
}

impl LickDisplay { 
    fn new(id: i32, filename: String, learned: i32) -> Self {
        let ld = LickDisplay {
            id,
            filename,
            learned
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

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct Lick {
    id : i32,
    filename : String,
    learned : i32,
}

async fn index() -> IndexTemplate{
    return IndexTemplate;
}

async fn get_lick(pool: &MySqlPool, id: &i32) -> Result<Lick, anyhow::Error> {
    let tmp = format!("SELECT id,filename,learned FROM Licks where id={}", id);
    let lick = sqlx::query_as::<_, Lick>(
        &tmp,)
        .fetch_optional(pool)
        .await?;

    if let Some(lick) = lick {
        return Ok(lick);
    }
    else {
        return Err(anyhow!("Entry not found: {id}"));
    }
}

async fn db_get(Query(params): Query<DbQuery>, State(pool): State<MySqlPool>) -> Result<LickDisplay, AppError> {
    let id: i32 = params.id;
    let lick = get_lick(&pool, &id).await?;
    return Ok(LickDisplay::new(
        lick.id,
        lick.filename, 
        lick.learned));
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
        .route("/db", get(db_get))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
} 

