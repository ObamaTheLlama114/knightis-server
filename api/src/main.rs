mod api;
mod database;


#[tokio::main]
async fn main() {
    let database = database::init_db("".to_string()).await.unwrap();
    api::init_server(database).await;
}
