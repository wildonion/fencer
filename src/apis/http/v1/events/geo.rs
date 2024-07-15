


use actix_web::{delete, put};
use geo_types::Polygon;
use migration::ArrayType;
use models::event::UpdateGeoRequest;
use postgis::{ewkb::{self, EwkbRead}, twkb::LineString};
use sea_orm::{ConnectionTrait, QueryResult, Statement, Value};
use crate::models::event::GeofenceRequest;
pub use super::*;



#[derive(Clone, Deserialize, Serialize)]
pub struct GeoQuery{
    pub imei: Option<String>,
    pub geo_id: Option<i32>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetResult{
    pub points: serde_json::Value,
    pub id: i32,
}
 
#[delete("/geo/{geo_id}")]
pub(self) async fn delete_geo(
    req: HttpRequest,
    geo_id: web::Path<i32>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let del_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::DELETE_BY_GEO_ID, 
            [
                geo_id.to_owned().into()
            ]
        );

    match db.execute(del_stmt).await{
        Ok(exec_res) => {

            resp!{
                &[u8],
                &[],
                None, // metadata
                &format!("SUCCESS: record has been deleted with id: {}", geo_id.to_owned()),
                StatusCode::OK,
                None::<Cookie<'_>>,
            }

        },
        Err(e) => {
            use crate::error::{ErrorKind, RustackiErrorResponse};
            let error_content = &e.to_string();
            let error_content = error_content.as_bytes().to_vec();
            let mut error_instance = RustackiErrorResponse::new(
                *consts::STORAGE_IO_ERROR_CODE, // error code
                error_content, // error content
                ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                "store_geo.db.execute", // method
                Some(&zerlog_producer_actor)
            ).await;

            return Err(error_instance); // terminate the caller
        }
    }

}


#[put("/geo/")]
pub(self) async fn update_geo(
    req: HttpRequest,
    new_geo: web::Json<UpdateGeoRequest>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let new_geo = new_geo.to_owned();
    let points: Vec<(f64, f64)> = new_geo.clone()
        .points
        .into_iter()
        .map(|p| (p.latitude, p.longitude))
        .collect();
    let json_points = serde_json::to_value(&points).unwrap();


    let geom_points: Vec<String> = new_geo
        .points
        .iter()
        .map(|p| format!("{} {}", p.latitude, p.longitude))
        .collect();

    let polygon = format!("POLYGON(({}))", geom_points.join(", "));

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let del_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::UPDATE_BY_GEO_ID, 
            [
                new_geo.geo_id.to_owned().into(),
                json_points.into(),
                polygon.into()
            ]
        );

    match db.execute(del_stmt).await{
        Ok(exec_res) => {

            resp!{
                &[u8],
                &[],
                None, // metadata
                &format!("SUCCESS: record has been updated"),
                StatusCode::OK,
                None::<Cookie<'_>>,
            }

        },
        Err(e) => {
            use crate::error::{ErrorKind, RustackiErrorResponse};
            let error_content = &e.to_string();
            let error_content = error_content.as_bytes().to_vec();
            let mut error_instance = RustackiErrorResponse::new(
                *consts::STORAGE_IO_ERROR_CODE, // error code
                error_content, // error content
                ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                "store_geo.db.execute", // method
                Some(&zerlog_producer_actor)
            ).await;

            return Err(error_instance); // terminate the caller
        }
    }
}



#[post("/geo/")]
pub(self) async fn store_geo(
    req: HttpRequest,
    geo_info: web::Json<GeofenceRequest>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let geo_info = geo_info.to_owned();
    let imei = geo_info.clone().imei;

    let points: Vec<(f64, f64)> = geo_info.clone()
        .points
        .into_iter()
        .map(|p| (p.latitude, p.longitude))
        .collect();
    let json_points = serde_json::to_value(&points).unwrap();


    let geom_points: Vec<String> = geo_info
        .points
        .iter()
        .map(|p| format!("{} {}", p.latitude, p.longitude))
        .collect();

    let polygon = format!("POLYGON(({}))", geom_points.join(", "));

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let insert_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::INSERT_QUERY, 
            [
                imei.into(),
                json_points.into(),
                polygon.into()
            ]
         );

     match db.execute(insert_stmt).await{
         Ok(exec_res) => {

            resp!{
                &[u8],
                &[],
                None, // metadata
                &format!("SUCCESS: record has been added"),
                StatusCode::OK,
                None::<Cookie<'_>>,
            }

         },
         Err(e) => {
             use crate::error::{ErrorKind, RustackiErrorResponse};
             let error_content = &e.to_string();
             let error_content = error_content.as_bytes().to_vec();
             let mut error_instance = RustackiErrorResponse::new(
                 *consts::STORAGE_IO_ERROR_CODE, // error code
                 error_content, // error content
                 ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                 "store_geo.db.execute", // method
                 Some(&zerlog_producer_actor)
             ).await;

             return Err(error_instance); // terminate the caller
         }
     }

}


// /geo/?geo_id=1
// /geo/?imei=""
#[get("/geo/")]
pub(self) async fn get_geo(
    req: HttpRequest,
    geo_q: web::Query<GeoQuery>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{


    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let geo_q = geo_q.0;
    let geo_q_imei = geo_q.imei;
    let get_g_id = geo_q.geo_id;


    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let query = if geo_q_imei.is_some(){
        Statement::from_sql_and_values(db_backend, 
            consts::queries::GET_QUERY_BY_GEO_IMEI, 
            [
                geo_q_imei.unwrap_or_default().into(),
            ]
        )
    } else if get_g_id.is_some(){
        Statement::from_sql_and_values(db_backend, 
            consts::queries::GET_QUERY_BY_GEO_ID, 
            [
                get_g_id.unwrap_or_default().into(),
            ]
        )
    } else{
        resp!{
            &[u8],
            &[],
            None, // metadata
            &format!("ERROR: there must be either imei or geo id"),
            StatusCode::NOT_ACCEPTABLE,
            None::<Cookie<'_>>,
        }
    };

     match db.query_all(query).await{
         Ok(data) => {

            let mut devices = vec![];
            for d in data{
                devices.push(
                    GetResult{
                        id: d.try_get("geo_id", "").unwrap(),
                        points: d.try_get("device_points", "").unwrap()
                    }
                )
            }

            resp!{
                Vec<GetResult>,
                devices,
                None, // metadata
                &format!("SUCCESS: fetched"),
                StatusCode::OK,
                None::<Cookie<'_>>,
            }

         },
         Err(e) => {
             use crate::error::{ErrorKind, RustackiErrorResponse};
             let error_content = &e.to_string();
             let error_content = error_content.as_bytes().to_vec();
             let mut error_instance = RustackiErrorResponse::new(
                 *consts::STORAGE_IO_ERROR_CODE, // error code
                 error_content, // error content
                 ErrorKind::Storage(crate::error::StorageError::SeaOrm(e)), // error kind
                 "get_geo.db.execute", // method
                 Some(&zerlog_producer_actor)
             ).await;

             return Err(error_instance); // terminate the caller
         }
     }

}



pub mod exports{
    pub use super::store_geo;
    pub use super::get_geo;
    pub use super::delete_geo;
    pub use super::update_geo;
}