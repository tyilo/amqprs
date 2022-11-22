use amqprs::{
    api::{
        channel::{
            BasicAckArguments, BasicGetArguments, BasicPublishArguments, QueueBindArguments,
            QueueDeclareArguments,
        },
        connection::{Connection, OpenConnectionArguments},
    },
    BasicProperties,
};
use tokio::time;
use tracing::{info, Level};
mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get() {
    let _guard = common::setup_logging(Level::DEBUG);

    // open a connection to RabbitMQ server
    let args = OpenConnectionArguments::new("localhost:5672", "user", "bitnami");

    let connection = Connection::open(&args).await.unwrap();

    // open a channel on the connection
    let mut channel = connection.open_channel().await.unwrap();

    let exchange_name = "amq.topic";
    // declare a queue
    let queue_name = "amqprs";
    channel
        .queue_declare(QueueDeclareArguments::new(queue_name))
        .await
        .unwrap();

    // bind the queue to exchange
    channel
        .queue_bind(QueueBindArguments::new(
            queue_name,
            exchange_name,
            "eiffel.#",
        ))
        .await
        .unwrap();

    // get empty
    let mut get_args = BasicGetArguments::new();
    get_args.queue = queue_name.to_string();

    let get_message = channel.basic_get(get_args.clone()).await.unwrap();
    if let Some(_) = get_message {
        panic!("expect ReturnEmpty message");
    }

    // contents to publish
    let content = String::from(
        r#"
            {
                "meta": {"id": "f9d42464-fceb-4282-be95-0cd98f4741b0", "type": "PublishTester", "version": "4.0.0", "time": 1640035100149},
                "data": { "customData": []}, 
                "links": [{"type": "BASE", "target": "fa321ff0-faa6-474e-aa1d-45edf8c99896"}]
            }
        "#
        ).into_bytes();

    // create arguments for basic_publish
    let mut args = BasicPublishArguments::new();
    // set target exchange name
    args.exchange = exchange_name.to_string();
    args.routing_key = "eiffel.a.b.c.d".to_string();

    let num_of_msg = 3;
    for _ in 0..num_of_msg {
        channel
            .basic_publish(BasicProperties::default(), content.clone(), args.clone())
            .await
            .unwrap();
    }

    for _ in 0..num_of_msg {
        // get single message
        let get_message = channel.basic_get(get_args.clone()).await.unwrap();
        let msg = match get_message {
            Some(msg) => {
                info!("Get a message: {:?}.", msg);
                msg
            }
            None => panic!("expect get a message"),
        };
        channel
            .basic_ack(BasicAckArguments {
                delivery_tag: msg.get_ok.delivery_tag(),
                multiple: false,
            })
            .await
            .unwrap();
    }
    // TODO: move to separate test case, below is for test only
    if true {
        // implicitly close by drop
        drop(channel);
        drop(connection);
    } else {
        // explicitly close
        channel.close().await.unwrap();
        connection.close().await.unwrap();
    }
    time::sleep(time::Duration::from_secs(1)).await;
}
