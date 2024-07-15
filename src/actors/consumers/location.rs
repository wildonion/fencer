


use actix::prelude::*;
use lapin::protocol::exchange;
use sea_orm::ConnectionTrait;
use uuid::Uuid;
use crate::actors::cqrs::accessors::location::CheckGeoFenceDbResponse;
use crate::actors::cqrs::accessors::location::CheckRouteDbResponse;
use crate::actors::cqrs::mutators::location::LocationMutatorActor;
use crate::actors::cqrs::mutators::location::StoreLocationEvent;
use crate::actors::producers::location::LocationProducerActor;
use crate::actors::producers::location::ProduceNotif;
use crate::actors::producers::zerlog::ZerLogProducerActor;
use crate::actors::cqrs::accessors::location::LocationAccessorActor;
use crate::models::event::{LocationEvent, LocationEventMessage};
use redis::{AsyncCommands, RedisResult};
use std::error::Error;
use std::sync::Arc;
use actix::{Actor, AsyncContext, Context};
use async_std::stream::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{message, BasicProperties};
use crate::plugins::notif::NotifExt;
use crate::s3::Storage;
use crate::consts::{self, MAILBOX_CHANNEL_ERROR_CODE, PING_INTERVAL};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ReceiverNotif{
    receiver_info: ReceiverInfo,
    notifs: Vec<NotifData>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ReceiverInfo{
    pub id: i32, // a unique identity
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum ActionType{ // all the action type that causes the notif to get fired
    #[default]
    Zerlog,
    GeoCheck
    // probably other system notifs
    // to be sent through SSE 
    // ...
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct NotifData{
    pub id: String,
    pub action_data: Option<serde_json::Value>, // any data
    pub actioner_info: Option<String>, // json stringified identifer
    pub action_type: ActionType, // type event
    pub fired_at: Option<i64>, 
    pub is_seen: bool,
}


#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct ConsumeNotif{
    /* -ˋˏ✄┈┈┈┈ 
        following queue gets bounded to the passed in exchange type with its 
        routing key, when producer wants to produce notif data it sends them 
        to the exchange with a known routing key, any queue that is bounded 
        to that exchange routing key will be filled up with messages coming 
        from the producer and they stay in there until a consumer read them
    */
    pub queue: String,
    pub exchange_name: String,
    /* -ˋˏ✄┈┈┈┈ 
        pattern for the exchange, any queue that is bounded to this exchange 
        routing key receives the message enables the consumer to consume the 
        message
    */
    pub routing_key: String,
    pub tag: String,
    pub redis_cache_exp: u64,
}

#[derive(Clone)]
pub struct LocationConsumerActor{
    pub app_storage: std::option::Option<Arc<Storage>>,
    pub notif_producer_actor: Addr<LocationProducerActor>,
    pub zerlog_producer_actor: Addr<ZerLogProducerActor>
}

impl Actor for LocationConsumerActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        log::info!("🎬 LocationConsumerActor has started, let's consume baby!");

        ctx.run_interval(PING_INTERVAL, |actor, ctx|{
            
            let this = actor.clone();

            tokio::spawn(async move{

                // check something constantly, schedule to be executed 
                // at a certain time in the background
                // ...
                
            });

        });

    }
}

impl LocationConsumerActor{

    pub fn new(app_storage: std::option::Option<Arc<Storage>>, 
            notif_producer_actor: Addr<LocationProducerActor>,
            zerlog_producer_actor: Addr<ZerLogProducerActor>) -> Self{
        Self { app_storage, notif_producer_actor, zerlog_producer_actor}
    }

    pub async fn consume(&self, exp_seconds: u64,
        consumer_tag: &str, queue: &str, 
        routing_key: &str, exchange: &str
    ){

        let storage = self.app_storage.clone();
        let rmq_pool = storage.clone().unwrap().get_lapin_pool().await.unwrap();
        let redis_pool = storage.clone().unwrap().get_redis_pool().await.unwrap();
        let notif_producer_actor = self.notif_producer_actor.clone();
        let zerlog_producer_actor = self.clone().zerlog_producer_actor;

        match rmq_pool.get().await{
            Ok(conn) => {

                let create_channel = conn.create_channel().await;
                match create_channel{
                    Ok(chan) => {

                        // -ˋˏ✄┈┈┈┈ making a queue inside the broker per each consumer, 
                        let create_queue = chan
                            .queue_declare(
                                &queue,
                                QueueDeclareOptions::default(),
                                FieldTable::default(),
                            )
                            .await;

                        let Ok(q) = create_queue else{
                            let e = create_queue.unwrap_err();
                            use crate::error::{ErrorKind, RustackiErrorResponse};
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();
                            let mut error_instance = RustackiErrorResponse::new(
                                *consts::STORAGE_IO_ERROR_CODE, // error code
                                error_content, // error content
                                ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                "LocationConsumerActor.queue_declare", // method
                                Some(&zerlog_producer_actor)
                            ).await;

                            return; // cancel streaming over consumer and terminate the caller
                        };

                        match chan
                            .queue_bind(q.name().as_str(), &exchange, &routing_key, 
                                QueueBindOptions::default(), FieldTable::default()
                            )
                            .await
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    use crate::error::{ErrorKind, RustackiErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = RustackiErrorResponse::new(
                                        *consts::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "LocationConsumerActor.queue_bind", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;

                                    return; // cancel streaming over consumer and terminate the caller
                                }
                            }
                    

                        // since &str is not lived long enough to be passed to the tokio spawn
                        // if it was static it would be good however we're converting them to
                        // String to pass the String version of them to the tokio spawn scope
                        let cloned_consumer_tag = consumer_tag.to_string();
                        let cloned_queue = queue.to_string();
                        tokio::spawn(async move{

                            // -ˋˏ✄┈┈┈┈ consuming from the queue owned by this consumer
                            match chan
                                .basic_consume(
                                    // the queue that is bounded to the exchange to receive messages based on the routing key
                                    // since the queue is already bounded to the exchange and its routing key it only receives 
                                    // messages from the exchange that matches and follows the passed in routing pattern like:
                                    // message routing key "orders.processed" might match a binding with routing key "orders.#
                                    // if none the messages follow the pattern then the queue will receive no message from the 
                                    // exchange based on that pattern!
                                    &cloned_queue, 
                                    &cloned_consumer_tag, // custom consumer name
                                    BasicConsumeOptions::default(), 
                                    FieldTable::default()
                                )
                                .await
                            {
                                Ok(mut consumer) => {

                                    // stream over consumer to receive data from the queue
                                    while let Some(delivery) = consumer.next().await{
                                        match delivery{
                                            Ok(delv) => {

                                                // if the consumer receives the data
                                                match delv.ack(BasicAckOptions::default()).await{
                                                    Ok(ok) => {

                                                        let buffer = delv.data;
                                                        let data = std::str::from_utf8(&buffer).unwrap();

                                                        log::info!("delivery data :::: {}", data);

                                                        let get_location_event = serde_json::from_str::<LocationEvent>(&data);
                                                        match get_location_event{
                                                            Ok(location_event) => {

                                                                let message = location_event.message;

                                                                log::info!("location event message :::: {:?}", message.clone());

                                                                let redis_notif_key = message.clone().imei.unwrap_or_default();

                                                                // caching in redis
                                                                match redis_pool.get().await{
                                                                    Ok(mut redis_conn) => {

                                                                        // -ˋˏ✄┈┈┈┈ notif pattern in reids
                                                                        // key  : String::from(device_imei)
                                                                        // value: Vec<LocationEventMessage>
                                                                        let get_device_events: RedisResult<String> = redis_conn.get(&redis_notif_key).await;
                                                                        let events = match get_device_events{
                                                                            Ok(events_string) => {
                                                                                let get_messages = serde_json::from_str::<Vec<LocationEventMessage>>(&events_string);
                                                                                match get_messages{
                                                                                    Ok(mut messages) => {
                                                                                        messages.push(message.clone());
                                                                                        messages
                                                                                    },
                                                                                    Err(e) => {
                                                                                        use crate::error::{ErrorKind, RustackiErrorResponse};
                                                                                        let error_content = &e.to_string();
                                                                                        let error_content_ = error_content.as_bytes().to_vec();
                                                                                        let mut error_instance = RustackiErrorResponse::new(
                                                                                            *consts::CODEC_ERROR_CODE, // error code
                                                                                            error_content_, // error content
                                                                                            ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                                            "LocationConsumerActor.decode_serde_redis", // method
                                                                                            Some(&zerlog_producer_actor)
                                                                                        ).await;

                                                                                        return; // cancel streaming over consumer and terminate the caller
                                                                                    }
                                                                                }
                                                                
                                                                            },
                                                                            Err(e) => {
                                                                                
                                                                                // we can't get the key means this is the first time we're creating the key
                                                                                // or the key is expired already, we'll create a new key either way and put
                                                                                // the init message in there.
                                                                                let init_message = vec![
                                                                                    message.clone()
                                                                                ];

                                                                                init_message

                                                                            }
                                                                        };


                                                                        /////// ------------------------------------------------------------------------------------------------------
                                                                        /////// ------------------------------ check the incoming locations is inside the route then isnert in db
                                                                        /////// ------------------------------------------------------------------------------------------------------

                                                                        let route_check_resp = LocationAccessorActor::check_route(
                                                                            message.clone(), storage.clone(), zerlog_producer_actor.clone()).await;

                                                                        log::info!("route_check_resp ::::: {:?}", route_check_resp);

                                                                        // getting the last entry from redis to check that 
                                                                        // the is_contains has changed or not if it has changed 
                                                                        // we'll insert into db and publish as notif to rmq
                                                                        let last_db_data = route_check_resp.clone().unwrap_or_default();
                                                                        let redis_notif_key = format!("route_{}", message.clone().imei.unwrap_or_default());
                                                                        let is_key_there: bool = redis_conn.exists(&redis_notif_key).await.unwrap();

                                                                        let mut should_we_insert = false;
                                                                        if is_key_there{
                                                                            let geo_check_data_string: String = redis_conn.get(&redis_notif_key).await.unwrap();
                                                                            let redis_last_data = serde_json::from_str::<CheckRouteDbResponse>(&geo_check_data_string).unwrap();
                                                                            if redis_last_data.is_contains != last_db_data.is_contains{ // outside of the route 
                                                                                should_we_insert = true;
                                                                            }
                                                                        } else{
                                                                            should_we_insert = true;
                                                                        };
                                                                        

                                                                        let stringified_route_check_resp = serde_json::to_string(&route_check_resp.clone().unwrap_or_default()).unwrap();
                                                                        let _: () = redis_conn.set(&redis_notif_key.clone(), &stringified_route_check_resp).await.unwrap();

                                                                        if should_we_insert{
                                                                            // store geo results
                                                                            let geo_result_store_resp = LocationMutatorActor::store_route_result(
                                                                                route_check_resp.clone(), zerlog_producer_actor.clone(),
                                                                                notif_producer_actor.clone(), storage.clone()
                                                                            ).await;
                                                                        }
                                                                        

                                                                        /////// ------------------------------------------------------------------------------------------------------
                                                                        /////// ------------------------------ check the incoming locations is inside the geofence then isnert in db
                                                                        /////// ------------------------------------------------------------------------------------------------------
                                                                        // check geofence process
                                                                        let geo_check_resp = LocationAccessorActor::check_geofence(
                                                                            message.clone(), storage.clone(), zerlog_producer_actor.clone()).await;

                                                                        log::info!("geo_check_resp ::::: {:?}", geo_check_resp);
                                                                        
                                                                        // getting the last entry from redis to check that 
                                                                        // the is_contains has changed or not if it has changed 
                                                                        // we'll insert into db and publish as notif to rmq
                                                                        let last_db_data = geo_check_resp.clone().unwrap_or_default();
                                                                        let redis_notif_key = format!("geo_{}", message.clone().imei.unwrap_or_default());
                                                                        let is_key_there: bool = redis_conn.exists(&redis_notif_key).await.unwrap();

                                                                        let mut should_we_insert = false;
                                                                        if is_key_there{
                                                                            log::info!("key is there for imei: [{}] at time: {}", message.clone().imei.unwrap(), chrono::Local::now().to_string());
                                                                            let geo_check_data_string: String = redis_conn.get(&redis_notif_key).await.unwrap();
                                                                            let redis_last_data = serde_json::from_str::<CheckGeoFenceDbResponse>(&geo_check_data_string).unwrap();
                                                                            if redis_last_data.is_contains != last_db_data.is_contains{ // outside of the fence
                                                                                should_we_insert = true;
                                                                            }
                                                                        } else{
                                                                            log::info!("key is not there for imei: {} at time: {}", message.clone().imei.unwrap(), chrono::Local::now().to_string());
                                                                            should_we_insert = true;
                                                                        };
                                                                        

                                                                        let stringified_geo_check_resp = serde_json::to_string(&geo_check_resp.clone().unwrap_or_default()).unwrap();
                                                                        let _: () = redis_conn.set(&redis_notif_key.clone(), &stringified_geo_check_resp).await.unwrap();

                                                                        if should_we_insert{
                                                                            // store geo results
                                                                            let geo_result_store_resp = LocationMutatorActor::store_geo_result(
                                                                                geo_check_resp.clone(), zerlog_producer_actor.clone(),
                                                                                notif_producer_actor.clone(), storage.clone()
                                                                            ).await;
                                                                        }


                                                                    },
                                                                    Err(e) => {
                                                                        use crate::error::{ErrorKind, RustackiErrorResponse};
                                                                        let error_content = &e.to_string();
                                                                        let error_content_ = error_content.as_bytes().to_vec();
                                                                        let mut error_instance = RustackiErrorResponse::new(
                                                                            *consts::STORAGE_IO_ERROR_CODE, // error code
                                                                            error_content_, // error content
                                                                            ErrorKind::Storage(crate::error::StorageError::RedisPool(e)), // error kind
                                                                            "LocationConsumerActor.redis_pool", // method
                                                                            Some(&zerlog_producer_actor)
                                                                        ).await;
                                                                        return; // cancel streaming over consumer and terminate the caller
                                                                    }
                                                                }

                                                            },
                                                            Err(e) => {
                                                                use crate::error::{ErrorKind, RustackiErrorResponse};
                                                                let error_content = &e.to_string();
                                                                let error_content_ = error_content.as_bytes().to_vec();
                                                                let mut error_instance = RustackiErrorResponse::new(
                                                                    *consts::CODEC_ERROR_CODE, // error code
                                                                    error_content_, // error content
                                                                    ErrorKind::Codec(crate::error::CodecError::Serde(e)), // error kind
                                                                    "LocationConsumerActor.decode_serde", // method
                                                                    Some(&zerlog_producer_actor)
                                                                ).await;

                                                                return; // cancel streaming over consumer and terminate the caller
                                                            }
                                                        }
                                                    },
                                                    Err(e) => {
                                                        use crate::error::{ErrorKind, RustackiErrorResponse};
                                                        let error_content = &e.to_string();
                                                        let error_content = error_content.as_bytes().to_vec();
                                                        let mut error_instance = RustackiErrorResponse::new(
                                                            *consts::STORAGE_IO_ERROR_CODE, // error code
                                                            error_content, // error content
                                                            ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                            "LocationConsumerActor.consume_ack", // method
                                                            Some(&zerlog_producer_actor)
                                                        ).await;

                                                        return; // cancel streaming over consumer and terminate the caller
                                                    }
                                                }
                    
                                            },
                                            Err(e) => {
                                                use crate::error::{ErrorKind, RustackiErrorResponse};
                                                let error_content = &e.to_string();
                                                let error_content = error_content.as_bytes().to_vec();
                                                let mut error_instance = RustackiErrorResponse::new(
                                                    *consts::STORAGE_IO_ERROR_CODE, // error code
                                                    error_content, // error content
                                                    ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                                    "LocationConsumerActor.consume_getting_delivery", // method
                                                    Some(&zerlog_producer_actor)
                                                ).await;

                                                return; // cancel streaming over consumer and terminate the caller 
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    use crate::error::{ErrorKind, RustackiErrorResponse};
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();
                                    let mut error_instance = RustackiErrorResponse::new(
                                        *consts::STORAGE_IO_ERROR_CODE, // error code
                                        error_content, // error content
                                        ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                                        "LocationConsumerActor.consume_basic_consume", // method
                                        Some(&zerlog_producer_actor)
                                    ).await;

                                    return; // cancel streaming over consumer and terminate the caller 
                                }
                            }

                        });

                    },
                    Err(e) => {
                        use crate::error::{ErrorKind, RustackiErrorResponse};
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
                        let mut error_instance = RustackiErrorResponse::new(
                            *consts::STORAGE_IO_ERROR_CODE, // error code
                            error_content, // error content
                            ErrorKind::Storage(crate::error::StorageError::Rmq(e)), // error kind
                            "LocationConsumerActor.consume_create_channel", // method
                            Some(&zerlog_producer_actor)
                        ).await;

                        return; // cancel streaming over consumer and terminate the caller   
                    }
                }

            },
            Err(e) => {
                use crate::error::{ErrorKind, RustackiErrorResponse};
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();
                let mut error_instance = RustackiErrorResponse::new(
                    *consts::STORAGE_IO_ERROR_CODE, // error code
                    error_content, // error content
                    ErrorKind::Storage(crate::error::StorageError::RmqPool(e)), // error kind
                    "LocationConsumerActor.consume_pool", // method
                    Some(&zerlog_producer_actor)
                ).await;

                return; // cancel streaming over consumer and terminate the caller
            }
        };

    }

}

impl Handler<ConsumeNotif> for LocationConsumerActor{
    
    type Result = ();
    fn handle(&mut self, msg: ConsumeNotif, ctx: &mut Self::Context) -> Self::Result {

        // unpacking the consume data
        let ConsumeNotif { 
                queue, 
                tag,
                exchange_name,
                routing_key,
                redis_cache_exp,
            } = msg.clone(); // the unpacking pattern is always matched so if let ... is useless
        
        let this = self.clone();
        tokio::spawn(async move{
            this.consume(redis_cache_exp, &tag, &queue, &routing_key, &exchange_name).await;
        });
        
        return; // cancel streaming over consumer and terminate the caller
    }

}