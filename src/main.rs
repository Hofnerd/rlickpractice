use anyhow::{anyhow, Context};
use askama_axum::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use dotenv::dotenv;
use rand::Rng;
use serde::Deserialize;
use sqlx::MySqlPool;
use tokio::fs::{self, remove_file};
//use tower::ServiceBuilder;
//use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;
//use tower_http::trace::TraceLayer;
//use tower_livereload::LiveReloadLayer;
//use tracing::Level;

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
#[template(path = "lick_pdf.html")]
pub struct PdfDisplay {
    pub lick: Lick,
}

impl PdfDisplay {
    fn new(in_lick: Lick) -> Self {
        let pd = PdfDisplay { lick: in_lick };
        return pd;
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    pub licks: Vec<Lick>,
}

impl IndexTemplate {
    fn new(licks: Vec<Lick>) -> Self {
        return IndexTemplate { licks };
    }
}

#[derive(Template)]
#[template(path = "lick.html")]
struct LickTemplate {
    pub lick: Lick,
}

impl LickTemplate {
    fn new(lick: Lick) -> Self {
        return LickTemplate { lick };
    }
}

#[derive(Template)]
#[template(path = "empty_pdf.html")]
struct EmptyPdfTemplate {}

#[derive(Deserialize)]
pub struct DbQuery {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct DbAdd {
    pub file: String,
}

#[derive(TryFromMultipart, Clone, Debug)]
struct PdfForm {
    name: String,
    pdf: axum::body::Bytes,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct Lick {
    id: i32,
    name: String,
    filename: String,
    learned: i32,
}

async fn index(State(pool): State<MySqlPool>) -> Result<IndexTemplate, AppError> {
    let licks = get_licks(&pool).await?;
    return Ok(IndexTemplate::new(licks));
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

async fn list_licks(State(pool): State<MySqlPool>) -> Result<IndexTemplate, AppError> {
    let licks = get_licks(&pool).await?;
    return Ok(IndexTemplate::new(licks));
}

async fn grab_file(pool: &MySqlPool, id: i32) -> Result<Lick, anyhow::Error> {
    let tmp: String = format!("SELECT id,name,filename,learned from Licks where id={}", id);
    let lick: Lick = sqlx::query_as(&tmp).fetch_one(pool).await?;
    return Ok(lick);
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

    let tmp = format!(
        "SELECT id,name,filename,learned FROM Licks where filename=\'{}\'",
        filename
    );
    let id: Lick = sqlx::query_as(&tmp).fetch_one(pool).await?;

    return Ok(id.id);
}

async fn check_incoming_pdf(
    pool: &MySqlPool,
    data: &TypedMultipart<PdfForm>,
) -> Result<bool, AppError> {
    let filename = format!("./data/{}.pdf", data.name);
    let query = format!("select id from Licks where filename=\"{}\"", filename);
    println!("{}", &query);
    let _ = match sqlx::query(&query).fetch_one(pool).await {
        Ok(_) => return Ok(false),
        Err(_) => return Ok(true),
    };
}

async fn upload_lick_pdf(
    State(pool): State<MySqlPool>,
    data: TypedMultipart<PdfForm>,
) -> Result<LickTemplate, AppError> {
    let res = check_incoming_pdf(&pool, &data).await?;
    if res {
        let filename = format!("./data/{}.pdf", data.name.replace(" ", "_"));
        let _ = fs::write(&filename, data.pdf.clone()).await?;
        let id = add_lick_db(&pool, &data.name, &filename).await?;
        return Ok(LickTemplate::new(get_lick(&pool, id).await?));
    }

    return Err(anyhow!(StatusCode::NOT_ACCEPTABLE).into());
}

async fn serve_pdf(
    Query(params): Query<DbQuery>,
    State(pool): State<MySqlPool>,
) -> Result<PdfDisplay, AppError> {
    let mut lick = grab_file(&pool, params.id).await?;
    let file = lick.filename.split(".").collect::<Vec<&str>>()[1];
    let file = file.to_owned() + ".pdf";
    lick.filename = file;
    return Ok(PdfDisplay::new(lick));
}

async fn del_lick(pool: &MySqlPool, id: i32) -> Result<i32, anyhow::Error> {
    let tmp: String = format!("delete FROM Licks where id ={id}");
    let _ = sqlx::query(&tmp).execute(pool).await?;
    Ok(1)
}

async fn delete_entry(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<IndexTemplate, AppError> {
    let lick = match get_lick(&pool, id).await {
        Ok(lick) => lick,
        Err(_) => {
            let list = get_licks(&pool).await?;
            return Ok(IndexTemplate::new(list));
        }
    };
    let _ = del_lick(&pool, id).await?;
    let _ = remove_file(lick.filename).await?;
    let list = get_licks(&pool).await?;
    return Ok(IndexTemplate::new(list));
}

async fn grab_random_file(pool: &MySqlPool) -> Result<Lick, anyhow::Error> {
    let id_query = format!("SELECT id,name,filename,learned from Licks");
    let ids: Vec<Lick> = sqlx::query_as(&id_query).fetch_all(pool).await?;
    let random_id = rand::thread_rng().gen_range(0..ids.len());
    let tmp: String = format!(
        "SELECT id,name,filename,learned from Licks where id={}",
        ids[random_id].id
    );
    let lick: Lick = sqlx::query_as(&tmp).fetch_one(pool).await?;
    return Ok(lick);
}

async fn random_lick(State(pool): State<MySqlPool>) -> Result<PdfDisplay, AppError> {
    let file = grab_random_file(&pool).await?;
    return Ok(PdfDisplay::new(file));
}

async fn set_learned(pool: &MySqlPool, id: i32) -> Result<i32, anyhow::Error> {
    let update_query = format!("UPDATE Licks set learned=1 where id={id}");
    sqlx::query(&update_query).execute(pool).await?;
    return Ok(id);
}

async fn set_entry_learned(
    State(pool): State<MySqlPool>,
    Path(id): Path<i32>,
) -> Result<LickTemplate, AppError> {
    let id = set_learned(&pool, id).await?;
    let lick = get_lick(&pool, id).await?;
    return Ok(LickTemplate::new(lick));
}

async fn close_pdf() -> Result<EmptyPdfTemplate, AppError> {
    return Ok(EmptyPdfTemplate {});
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use dotenv to load the .env file for the database url
    dotenv().ok();
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = sqlx::MySqlPool::connect(&db_connection_str)
        .await
        .context("cant connect to database")?;

    let assets_path = std::env::current_dir().unwrap();

    //let service = ServiceBuilder::new()
    //    .layer(TraceLayer::new_for_http())
    //    .layer(NormalizePathLayer::trim_trailing_slash())
    //    .layer(LiveReloadLayer::new());

    let app = Router::new()
        .route("/", get(index).post(upload_lick_pdf))
        .route("/lick/:id", delete(delete_entry).put(set_entry_learned))
        .route("/rand", get(random_lick))
        .route("/licks", get(list_licks))
        .route("/pdf", get(serve_pdf).put(close_pdf))
        .nest_service("/data", ServeDir::new("./data"))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        )
        //    .layer(service)
        .with_state(pool);

    //tracing_subscriber::fmt::Subscriber::builder()
    //    .with_max_level(Level::TRACE)
    //    .init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
