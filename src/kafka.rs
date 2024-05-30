use rdkafka::{producer::FutureProducer, ClientConfig};

pub async fn producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("allow.auto.create.topics", "true")
        .create()
        .expect("Producer creation error")
}
