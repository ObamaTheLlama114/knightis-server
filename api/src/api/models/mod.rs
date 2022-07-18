use std::{pin::Pin, time::Duration};

use juniper::{graphql_object, futures, graphql_subscription};

use super::context;

pub struct Query;
#[graphql_object(context = super::context)]
impl Query {
    fn api_version() -> &'static str {
        "1.0"
    }
}

pub struct Mutation;
#[graphql_object(context = super::context)]
impl Mutation {
    fn api_version() -> &'static str {
        "1.0"
    }
}

type RandomHumanStream =
    Pin<Box<dyn futures::Stream<Item = i32> + Send>>;
pub struct Subscription;
#[graphql_subscription(context = context)]
impl Subscription {
    #[graphql(
        description = "A random humanoid creature in the Star Wars universe every 3 seconds. Second result will be an error."
    )]
    async fn one() -> RandomHumanStream {

        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let stream = async_stream::stream! {
            loop {
                interval.tick().await;
                yield 1
            }
        };

        Box::pin(stream)
    }
}

