use sqlx::{Error, FromRow, MySql, MySqlPool, Pool};
use sqlx::types::chrono;
use sqlx::types::chrono::DateTime;
use async_std::task;

#[derive(Default)]
pub struct SqlReader { }

#[derive(FromRow,Clone)]
pub struct Lick {
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

    async fn get_lick_from_table(&self) -> Option<Vec<Lick>> {
        let result = task::block_on(self.connect());

        match result {
            Err(_err) => {
                return None;
            }
            Ok(pool) => {
                let query_result = sqlx::query_as::<_, Lick>("select * from Licks")
                    .fetch_all(&pool).await.unwrap();
                return Some(query_result);
            }
        }
    }

    pub fn test_connection(&self) {
        task::block_on(self.do_test_connection());
    }

    pub fn get_licks(&self) -> Option<Vec<Lick>>{
        let test = task::block_on(self.get_lick_from_table());
        match test.clone() {
            None => {
                println!("Unable to grab any values");
            }
            Some(vals) => {
                println!("Number of Licks selected: {}", vals.len());

                for i in vals {
                    println!("{} {} {} {} {}", i.id, i.filename, i.learned, i.date_completed, i.date_sub);
                }
            }
        }
        return test;
    }

    pub fn new() -> SqlReader {
        SqlReader { }
    }

}

impl Lick {
    pub fn new() -> Lick {
        Lick {
            id: 1,
            filename: "tmp".to_string(),
            learned: 0,
            date_completed: DateTime::UNIX_EPOCH,
            date_sub: DateTime::UNIX_EPOCH,
        }
    }

    pub fn get_id(&self) -> i32 {
        return self.id;
    }

    pub fn get_learned(&self) -> i32 {
        return self.learned;
    }
}