use sqlx::{postgres::PgPoolOptions, Pool, Error, Postgres};

pub async fn init_db(sql_url: String) -> Result<Pool<Postgres>, Error> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&sql_url).await?)
}