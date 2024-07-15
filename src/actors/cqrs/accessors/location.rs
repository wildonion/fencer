


use std::str::FromStr;
use std::sync::Arc;
use actix::{Actor, AsyncContext, Context, Handler};
use chrono::{DateTime, FixedOffset};
use deadpool_redis::{Connection, Manager, Pool};
use redis::{AsyncCommands, Commands};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Statement, Value};
use crate::actors::consumers;
use crate::models::event::LocationEventMessage;
use crate::types::RedisPoolConnection;
use crate::{models::event::FetchLocationBasicReport};
use crate::s3::Storage;
use crate::consts::PING_INTERVAL;
use actix::prelude::*;
use serde::{Serialize, Deserialize};
use crate::consts;


#[derive(Message, Clone, Serialize, Deserialize, Debug, Default)]
#[rtype(result = "()")]
pub struct RequestLocationBasicReport{
    pub device_imei: String,
    pub from: String,
    pub to: String
}

#[derive(Clone)]
pub struct LocationAccessorActor{
    pub app_storage: std::option::Option<Arc<Storage>>,
}


#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CheckGeoFenceDbResponse{
    pub device_info: LocationEventMessage,
    pub geo_id: i32,
    pub device_imei: String,
    pub geom_text: String,
    pub point_text: String, 
    pub is_contains: bool
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CheckRouteDbResponse{
    pub device_info: LocationEventMessage,
    pub route_id: i32,
    pub device_imei: String,
    pub route_text: String,
    pub point_text: String, 
    pub is_contains: bool
}

impl Actor for LocationAccessorActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        log::info!("🎬 LocationAccessorActor has started, let's read baby!");

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

impl LocationAccessorActor{

    pub fn new(app_storage: std::option::Option<Arc<Storage>>) -> Self{
        Self { app_storage }
    }
    

    pub async fn check_route(message: LocationEventMessage, storage: Option<Arc<Storage>>, 
        producer_actor: Addr<crate::actors::producers::zerlog::ZerLogProducerActor>) 
        -> Option<CheckRouteDbResponse>{

        let imei = message.clone().imei;
        let lat = message.latitude;
        let long = message.longitude;

        let db = storage.as_ref().unwrap().get_seaorm_pool().await.unwrap();

        let db_backend = db.get_database_backend();
        let retrieve_stmt = Statement::from_sql_and_values(db_backend,
            consts::queries::CHECK_ROUTE_QUERY, 
            [
                imei.into(),
                lat.unwrap_or_default().into(),
                long.unwrap_or_default().into(),
            ]
        );

        match db.query_one(retrieve_stmt).await{
            Ok(query_res) => {
                
                if query_res.is_none(){
                    return None;
                };
                let d = query_res.unwrap();
                let resp_check = CheckRouteDbResponse{
                    device_info: message.clone(),
                    route_id: d.try_get("", "id").unwrap(),
                    device_imei: d.try_get("", "imei").unwrap(),
                    route_text: d.try_get("geom_text", "").unwrap(),
                    point_text: d.try_get("point_text", "").unwrap(),
                    is_contains: d.try_get("is_contains", "").unwrap()
                };
                return Some(resp_check);
           
            },
            Err(e) => {
                use crate::error::{ErrorKind, RustackiErrorResponse};
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();
                let mut error_instance = RustackiErrorResponse::new(
                    *consts::STORAGE_IO_ERROR_CODE, // error code
                    error_content, // error content
                    ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                    "LocationAccessorActor.check_route.db.query_all", // method
                    Some(&producer_actor)
                ).await;
                
                return None;
            }
        }

    }

    pub async fn check_geofence(message: LocationEventMessage, storage: Option<Arc<Storage>>, 
        producer_actor: Addr<crate::actors::producers::zerlog::ZerLogProducerActor>) 
        -> Option<CheckGeoFenceDbResponse>{

        let imei = message.clone().imei;
        let lat = message.latitude;
        let long = message.longitude;

        let db = storage.as_ref().unwrap().get_seaorm_pool().await.unwrap();

        // executing report query on hypertable
        let db_backend = db.get_database_backend(); // postgres in our case
        let retrieve_stmt = Statement::from_sql_and_values(db_backend,
                consts::queries::CHECK_QUERY, 
                [
                    imei.into(),
                    lat.unwrap_or_default().into(),
                    long.unwrap_or_default().into(),
                ]
            );

        match db.query_one(retrieve_stmt).await{
            Ok(query_res) => {
                
                if query_res.is_none(){
                    return None;
                };
                let d = query_res.unwrap();
                let resp_check = CheckGeoFenceDbResponse{
                    device_info: message.clone(),
                    geo_id: d.try_get("", "id").unwrap(),
                    device_imei: d.try_get("", "imei").unwrap(),
                    geom_text: d.try_get("geom_text", "").unwrap(),
                    point_text: d.try_get("point_text", "").unwrap(),
                    is_contains: d.try_get("is_contains", "").unwrap()
                };
                return Some(resp_check);
           
            },
            Err(e) => {
                use crate::error::{ErrorKind, RustackiErrorResponse};
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();
                let mut error_instance = RustackiErrorResponse::new(
                    *consts::STORAGE_IO_ERROR_CODE, // error code
                    error_content, // error content
                    ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                    "LocationAccessorActor.check_geofence.db.query_all", // method
                    Some(&producer_actor)
                ).await;
                
                return None;
            }
        }
        
    }

}


// a dead handler!
impl Handler<RequestLocationBasicReport> for LocationAccessorActor{

    type Result = ();

    fn handle(&mut self, msg: RequestLocationBasicReport, ctx: &mut Self::Context) -> Self::Result {
        
        let RequestLocationBasicReport{
            device_imei,
            from,
            to
        } = msg;

        let this = self.clone();
        
        tokio::spawn(async move{
            
            // ...

        });

    }
    
}