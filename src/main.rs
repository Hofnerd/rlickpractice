use anyhow::Context;
use askama_axum::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::MySqlPool;
use tokio::fs::{self, remove_file};
use tower::ServiceBuilder;
use tower_http::{normalize_path::NormalizePathLayer, services::ServeDir, trace::TraceLayer};
use tracing::Level;

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
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        return Self(err.into());
    }
}

#[derive(Deserialize, Debug, Template)]
#[template(path = "licks.html")]
pub struct LicksDisplay {
    pub licks: Vec<Lick>,
}

impl LicksDisplay {
    fn new(licks: Vec<Lick>) -> Self {
        let ld = LicksDisplay { licks };
        return ld;
    }
}

#[derive(Deserialize, Debug, Template)]
#[template(path = "lick_pdf.html")]
pub struct PdfDisplay {
    pub pdf: String,
}

impl PdfDisplay {
    fn new(name: String) -> Self {
        let pd = PdfDisplay { pdf: name };
        return pd;
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
    id: i32,
    name: String,
    filename: String,
    learned: i32,
}

async fn index() -> IndexTemplate {
    return IndexTemplate;
}

async fn get_lick(pool: &MySqlPool, id: i32) -> Result<Lick, anyhow::Error> {
    let tmp: String = format!("SELECT id,name,filename,learned FROM Licks where id = {id}");
    let lick = sqlx::query_as(&tmp).fetch_one(pool).await?;
    Ok(lick)
}

async fn get_licks(pool: &MySqlPool) -> Result<Vec<Lick>, anyhow::Error> {
    let tmp: String = format!("SELECT id,name,filename,learned FROM Licks");
    let licks = sqlx::query_as(&tmp).fetch_all(pool).await?;
    Ok(licks)
}

async fn list_licks(State(pool): State<MySqlPool>) -> Result<LicksDisplay, AppError> {
    let licks = get_licks(&pool).await?;
    return Ok(LicksDisplay::new(licks));
}

async fn grab_file(pool: &MySqlPool, id: i32) -> Result<String, anyhow::Error> {
    let tmp: String = format!("SELECT id,name,filename,learned from Licks where id={}", id);
    let lick: Lick = sqlx::query_as(&tmp).fetch_one(pool).await?;
    return Ok(lick.filename);
}

async fn add_lick_db(
    pool: &MySqlPool,
    name: &String,
    filename: &String,
) -> Result<i32, anyhow::Error> {
    let tmp = format!(
        "INSERT INTO Licks (name,filename) values ('{}','{}')",
        name, filename
    );
    sqlx::query(&tmp).execute(pool).await?;
    return Ok(0);
}

async fn serve_pdf(
    Query(params): Query<DbQuery>,
    State(pool): State<MySqlPool>,
) -> Result<PdfDisplay, AppError> {
    let filename = grab_file(&pool, params.id).await?;
    let file = filename.split(".").collect::<Vec<&str>>()[1];
    let file = file.to_owned() + ".pdf";
    return Ok(PdfDisplay::new(file));
}

#[derive(TryFromMultipart, Clone, Debug)]
struct PdfForm {
    name: String,
    pdf: axum::body::Bytes,
}

async fn upload_lick_pdf(
    State(pool): State<MySqlPool>,
    data: TypedMultipart<PdfForm>,
) -> Result<IndexTemplate, AppError> {
    let filename = format!("./data/{}.pdf", data.name);
    let _ = fs::write(&filename, data.pdf.clone()).await?;
    let _ = add_lick_db(&pool, &data.name, &filename).await?;
    return Ok(IndexTemplate);
}

async fn del_lick(pool: &MySqlPool, id: i32) -> Result<i32, anyhow::Error> {
    let tmp: String = format!("delete FROM Licks where id ={id}");
    let _ = sqlx::query(&tmp).execute(pool).await?;
    Ok(1)
}

async fn delete_entry(
    State(pool): State<MySqlPool>,
    Query(params): Query<DbQuery>,
) -> Result<LicksDisplay, AppError> {
    let id: i32 = params.id;
    let lick = match get_lick(&pool, id).await {
        Ok(lick) => lick,
        Err(_) => {
            let list = get_licks(&pool).await?;
            return Ok(LicksDisplay::new(list));
        }
    };
    let _ = del_lick(&pool, id).await?;
    let _ = remove_file(lick.filename).await?;
    let list = get_licks(&pool).await?;
    return Ok(LicksDisplay::new(list));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use dotenv to load the .env file for the database url
    dotenv().ok();
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = sqlx::MySqlPool::connect(&db_connection_str)
        .await
        .context("cant connect to database")?;

    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(NormalizePathLayer::trim_trailing_slash());

    let app = Router::new()
        .route("/", get(index).post(upload_lick_pdf))
        .route("/dellick", get(delete_entry))
        .route("/licks", get(list_licks))
        .route("/pdf", get(serve_pdf))
        .layer(service)
        .nest_service("/data", ServeDir::new("./data"))
        .with_state(pool);

    // tracing_subscriber::fmt::Subscriber::builder()
    //     .with_max_level(Level::TRACE)
    //     .init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
