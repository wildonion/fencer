
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait, Statement, TryIntoModel, Value};
use serde::{Serialize, Deserialize};
use actix::prelude::*;
use uuid::Uuid;
use std::error::Error;
use std::sync::Arc;
use actix::{Actor, AsyncContext, Context};
use crate::actors::cqrs::accessors::location::{CheckGeoFenceDbResponse, CheckRouteDbResponse};
use crate::actors::producers::location::LocationProducerActor;
use crate::actors::producers::zerlog::ZerLogProducerActor;
use crate::actors::consumers::location::NotifData;
use crate::actors::producers::location::ProduceNotif;
use crate::actors::consumers::location::ActionType;
use crate::consts::MAILBOX_CHANNEL_ERROR_CODE;
use crate::entities::hoops;
use crate::models::event::LocationEventMessage;
use crate::s3::Storage;
use crate::consts::{self, PING_INTERVAL};
use serde_json::json;

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StoreLocationEvent{
    pub message: LocationEventMessage
}



#[derive(Clone)]
pub struct LocationMutatorActor{
    pub app_storage: std::option::Option<Arc<Storage>>,
    pub zerlog_producer_actor: Addr<ZerLogProducerActor>
}

impl Actor for LocationMutatorActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        log::info!("🎬 LocationMutatorActor has started, let's mutate baby!");

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

impl LocationMutatorActor{

    pub fn new(app_storage: std::option::Option<Arc<Storage>>, zerlog_producer_actor: Addr<ZerLogProducerActor>) -> Self{
        Self { app_storage, zerlog_producer_actor }
    }

    pub async fn store_geo_result(message: Option<CheckGeoFenceDbResponse>, 
        producer_actor: Addr<crate::actors::producers::zerlog::ZerLogProducerActor>,
        notif_producer_actor: Addr<LocationProducerActor>,
        storage: Option<Arc<Storage>>){
    
        let storage = storage.as_ref().clone().unwrap();
        let db = storage.get_seaorm_pool().await.unwrap();
        let data = message.clone().unwrap_or_default();

        let db_backend = db.get_database_backend();
        
            let cgr = message.clone().unwrap_or_default();
            let stmt = Statement::from_sql_and_values(
                db_backend, 
                consts::queries::INSERT_INTO_GEO_RESULT, 
                [
                    cgr.clone().geo_id.into(),
                    cgr.clone().device_imei.into(),
                    cgr.clone().geom_text.into(),
                    cgr.clone().point_text.into(),
                    cgr.clone().is_contains.into()
                ]
            );
    
            match db.execute(stmt).await{
                Ok(res) => {
                    
                    // if we're here means that the is_contains has changed
                    // publish the result of geofence checker as notif into rmq
                    tokio::spawn(
                        {
                            let cloned_producer_actor = notif_producer_actor.clone();
                            let zerlog_producer_actor = producer_actor.clone();
                            async move{
                                match cloned_producer_actor
                                    .send(
                                        ProduceNotif{
                                            notif_receiver: Default::default(),
                                            notif_data: NotifData{
                                                id: Uuid::new_v4().to_string(),
                                                action_data: {
                                                    Some(
                                                        serde_json::to_value(&cgr).unwrap()
                                                    )
                                                },
                                                actioner_info: Some(Uuid::new_v4().to_string()),
                                                action_type: ActionType::GeoCheck,
                                                fired_at: Some(chrono::Local::now().timestamp()),
                                                is_seen: false,
                                            },
                                            exchange_name: String::from("GeofenceExchange"),
                                            exchange_type: String::from("fanout"),
                                            routing_key: String::from(""),
                                        }
                                    )
                                    .await
                                    {
                                        Ok(_) => {},
                                        Err(e) => {
                                            let source = &e.source().unwrap().to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                                            let err_instance = crate::error::RustackiErrorResponse::new(
                                                *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                                source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                                crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                                &String::from("LocationMutatorActor.store_geo_result.db.execute"), // current method name
                                                Some(&zerlog_producer_actor)
                                            ).await;
                                            return;
                                        }
                                    }
                            }
                        }
                    );

                },
                Err(e) => {
                    use crate::error::{ErrorKind, RustackiErrorResponse};
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();
                    let mut error_instance = RustackiErrorResponse::new(
                        *consts::STORAGE_IO_ERROR_CODE, // error code
                        error_content, // error content
                        ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                        "LocationMutatorActor.store_geo_result.db.execute", // method
                        Some(&producer_actor)
                    ).await;
    
                    return; // terminate the caller
                }
            }
        }

        pub async fn store_route_result(message: Option<CheckRouteDbResponse>, 
            producer_actor: Addr<crate::actors::producers::zerlog::ZerLogProducerActor>,
            notif_producer_actor: Addr<LocationProducerActor>,
            storage: Option<Arc<Storage>>){
        
            let storage = storage.as_ref().clone().unwrap();
            let db = storage.get_seaorm_pool().await.unwrap();
            let data = message.clone().unwrap_or_default();
    
            let db_backend = db.get_database_backend();
            
                let cgr = message.clone().unwrap_or_default();
                let stmt = Statement::from_sql_and_values(
                    db_backend, 
                    consts::queries::INSERT_INTO_ROUTE_RESULT, 
                    [
                        cgr.clone().route_id.into(),
                        cgr.clone().device_imei.into(),
                        cgr.clone().route_text.into(),
                        cgr.clone().point_text.into(),
                        cgr.clone().is_contains.into()
                    ]
                );
        
                match db.execute(stmt).await{
                    Ok(res) => {
                        
                        // if we're here means that the is_contains has changed
                        // publish the result of route checker as notif into rmq
                        // later other subscribers or consumers can start subscribing to it 
                        // to take the notification
                        tokio::spawn( 
                            {
                                let cloned_producer_actor = notif_producer_actor.clone();
                                let zerlog_producer_actor = producer_actor.clone();
                                async move{
                                    match cloned_producer_actor
                                        .send(
                                            ProduceNotif{
                                                notif_receiver: Default::default(),
                                                notif_data: NotifData{
                                                    id: Uuid::new_v4().to_string(),
                                                    action_data: {
                                                        Some(
                                                            serde_json::to_value(&cgr).unwrap()
                                                        )
                                                    },
                                                    actioner_info: Some(Uuid::new_v4().to_string()),
                                                    action_type: ActionType::GeoCheck,
                                                    fired_at: Some(chrono::Local::now().timestamp()),
                                                    is_seen: false,
                                                },
                                                exchange_name: String::from("RouteExchange"),
                                                exchange_type: String::from("fanout"),
                                                routing_key: String::from(""),
                                            }
                                        )
                                        .await
                                        {
                                            Ok(_) => {},
                                            Err(e) => {
                                                let source = &e.source().unwrap().to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                                                let err_instance = crate::error::RustackiErrorResponse::new(
                                                    *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                                    source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                                    crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                                    &String::from("LocationMutatorActor.store_geo_result.db.execute"), // current method name
                                                    Some(&zerlog_producer_actor)
                                                ).await;
                                                return;
                                            }
                                        }
                                }
                            }
                        );
    
                    },
                    Err(e) => {
                        use crate::error::{ErrorKind, RustackiErrorResponse};
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
                        let mut error_instance = RustackiErrorResponse::new(
                            *consts::STORAGE_IO_ERROR_CODE, // error code
                            error_content, // error content
                            ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                            "LocationMutatorActor.store_geo_result.db.execute", // method
                            Some(&producer_actor)
                        ).await;
        
                        return; // terminate the caller
                    }
                }
            }
        
}