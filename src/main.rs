use dotenv::dotenv;
use anyhow::{anyhow, Context};
use axum::{
    extract::{Query, State}, http::StatusCode, routing::get, Router
};
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct DbQuery {
    pub index: i32,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct Lick {
    id : i32,
    filename : String,
    learned : i32,
}

async fn index() -> &'static str {
    return "Index";
}

async fn hello() -> &'static str {
    return "Hello world"
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

async fn db_get(Query(params): Query<DbQuery>, State(pool): State<MySqlPool>) -> Result<String, StatusCode> {
    let id: i32 = params.index;
    match get_lick(&pool, &id).await {
        Ok(lick) => Ok(format!(
            "{}: {} {}\n", lick.id, lick.filename, lick.learned)),
        Err(_) => Err(StatusCode::NO_CONTENT),
    }
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
        .route("/hello", get(hello))
        .route("/db", get(db_get))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())

} 

