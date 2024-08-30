use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info};

#[derive(Serialize, Deserialize, Debug)]
struct ItemOrder {
    item: String,
    price: f32,
    count: i16,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderMessage {
    user_id: usize,
    order_id: usize,
    items: Vec<ItemOrder>,
}
impl OrderMessage {
    fn create_confirmation_notice(&self) -> UserConfirmationMessage {
        let mut total_price = 0.0;
        let mut total_items = 0;
        for item in &self.items {
            total_price += item.price * f32::from(item.count);
            total_items += item.count;
        }
        let text = format!(
            "Thanks for your order, for ${} from {} items",
            total_price, total_items
        );

        return UserConfirmationMessage {
            user_id: self.user_id,
            text,
            total_price,
            total_items,
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UserConfirmationMessage {
    user_id: usize,
    text: String,
    total_price: f32,
    total_items: i16,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderProcessedMessage {
    order_id: usize,
    processed: bool,
}

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    tracing_subscriber::fmt().init();
    // Connect to the NATS server
    let client = async_nats::connect("nats://nats:4222").await?;

    // Subscribe to the "messages" subject
    let mut subscriber = client.subscribe("orders").await?;
    info!("Subscribed to orders");
    let range = 1..20;
    let orders: Vec<String> = range
        .map(|i| {
            let order = json!({
                "user_id": i,
                "order_id": i * 2,
                "items": [{
                    "item": "thing",
                    "price": 1.50,
                    "count": 1 + (i % 2)
                },{
                    "item": "thing2",
                    "price": 2.50,
                    "count": 1 + (i % 2)
                }]
            });
            order.to_string()
        })
        .collect();
    for o in orders {
        client.publish("orders", o.into()).await?;
    }

    // Receive and process messages
    while let Some(message) = subscriber.next().await {
        debug!(size = message.length, "Received message");
        //message.payload
        println!("Received message {:?}", message.payload);
        match serde_json::from_slice::<OrderMessage>(&message.payload.to_vec()) {
            Ok(order) => {
                let user_confirmation = order.create_confirmation_notice();
                let confirm_payload = serde_json::to_string(&user_confirmation)?;
                let process_message = match client
                    .publish("order_confirmation_email", Bytes::from(confirm_payload))
                    .await
                {
                    Ok(_) => OrderProcessedMessage {
                        order_id: order.order_id,
                        processed: true,
                    },
                    Err(_) => OrderProcessedMessage {
                        order_id: order.order_id,
                        processed: false,
                    },
                };
                let process_payload = serde_json::to_string(&process_message)?;
                client
                    .publish("processed.orders", Bytes::from(process_payload))
                    .await?
            }
            Err(e) => {
                error!("Failed to parse order message, error: {:?}", e);
            }
        }
    }

    Ok(())
}
