use sqlx::{Pool, MySql, Error, MySqlPool, FromRow};
use sqlx::types::chrono;
use sqlx::types::chrono::{DateTime};
use async_std::task;

#[derive(Default)]
pub struct SqlReader {
    test: u64,
}

#[derive(FromRow)]
struct Lick {
    id : i32,
    filename : String,
    learned : i32,
    date_comp : DateTime<chrono::Utc>,
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

            Ok(_) => {
                println!("Connected to database successfully.");

                let query_result = 
                    sqlx::query_as::<_, Lick>("select * from Licks where id='4'");
            }
        }
    }
    
    pub fn acquire_connection(&self) {
        task::block_on(self.do_test_connection());
    }

    pub fn new() -> SqlReader {
        SqlReader {
            test : 10
        }
    }

}