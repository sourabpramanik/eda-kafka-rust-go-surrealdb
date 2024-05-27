use rdkafka::{
    message::{Headers, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

pub async fn producer(brokers: &str, topic_name: &str) -> FutureProducer {
    &ClientConfig::new()
        .set("bootstrap.server", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}
