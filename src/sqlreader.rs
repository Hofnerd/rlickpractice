use sqlx::{Error, FromRow, MySql, MySqlPool, Pool};
use sqlx::types::chrono;
use sqlx::types::chrono::DateTime;
use async_std::task;

#[derive(Default)]
pub struct SqlReader { }

#[derive(FromRow)]
struct Lick {
    id : i32,
    filename : String,
    learned : i32,
    date_completed : DateTime<chrono::Utc>,
    date_sub : DateTime<chrono::Utc>,
}

impl SqlReader {

    async fn connect(&self) -> Result<Pool<MySql>, Error>{
        return MySqlPool::connect("mysql://sdlomba:password@localhost/lp").await;
    }
    
    async fn do_test_connection(&self) {
        let result = task::block_on(self.connect());
        match result {
            Err(err) => {
                println!("Cannot connect to database [{}]", err.to_string());
            }

            Ok(pool) => {
                println!("Connected to database successfully.");

                let query_result = sqlx::query_as::<_, Lick>("select * from Licks")
                    .fetch_all(&pool).await.unwrap();

                println!("Number of Licks selected: {}", query_result.len());

                for i in query_result {
                    println!("{} {} {} {} {}", i.id, i.filename, i.learned, i.date_completed, i.date_sub);
                }
            }
        }
    }
    
    pub fn acquire_connection(&self) {
        task::block_on(self.do_test_connection());
    }

    pub fn new() -> SqlReader {
        SqlReader { }
    }

}