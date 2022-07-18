use std::{env, sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{HttpResponse, Error, web::{self, Data}, http::header, middleware, HttpServer, App, HttpRequest};
use juniper::{RootNode, EmptyMutation, EmptySubscription, Context};
use juniper_actix::{graphiql_handler, playground_handler, graphql_handler, subscriptions::subscriptions_handler};
use juniper_graphql_ws::ConnectionConfig;
use sqlx::{Pool, Postgres};

pub mod models;


pub struct context {
    pub database: Arc<Pool<Postgres>>,
}

impl Context for context {}

type Schema = RootNode<'static, models::Query, EmptyMutation<context>, EmptySubscription<context>>;

fn schema() -> Schema {
    Schema::new(
        models::Query,
        EmptyMutation::<>::new(),
        EmptySubscription::<>::new(),
    )
}

async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", None).await
}
async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", None).await
}
async fn graphql_route(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    schema: web::Data<Schema>,
    database: web::Data<Pool<Postgres>>,

) -> Result<HttpResponse, Error> {
    graphql_handler(&schema, &context { database: database.into_inner() }, req, payload).await
}

async fn subscriptions(
    req: HttpRequest,
    stream: web::Payload,
    schema: web::Data<Schema>,
    database: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, Error> {
    let schema = schema.into_inner();
    let config = ConnectionConfig::new(context { database: database.into_inner() });
    // set the keep alive interval to 15 secs so that it doesn't timeout in playground
    // playground has a hard-coded timeout set to 20 secs
    let config = config.with_keep_alive_interval(Duration::from_secs(15));

    subscriptions_handler(req, stream, schema, config).await
}

pub async fn init_server(database: Pool<Postgres>) {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema()))
            .app_data(database.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/subscriptions").route(web::get().to(subscriptions)))
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
            .default_service(web::to(|| async {
                HttpResponse::Found()
                    .append_header((header::LOCATION, "/playground"))
                    .finish()
            }))
    });
    server.bind("127.0.0.1:8080").unwrap().run().await.unwrap();
}