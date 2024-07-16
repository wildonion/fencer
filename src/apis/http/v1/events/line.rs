


use actix_web::{delete, put};
use migration::ArrayType;
use models::event::{LineRequest, UpdateLineRequest};
use postgis::{ewkb::{self, EwkbRead}, twkb::LineString};
use sea_orm::{ConnectionTrait, QueryResult, Statement, Value};
pub use super::*;



#[derive(Clone, Deserialize, Serialize)]
pub struct LineQuery{
    pub imei: Option<String>,
    pub line_id: Option<i32>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetResult{
    pub points: serde_json::Value,
    pub id: i32,
}

#[delete("/route/{line_id}")]
pub(self) async fn delete_line(
    req: HttpRequest,
    line_id: web::Path<i32>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let del_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::DELETE_BY_ROUTE_ID, 
            [
                line_id.to_owned().into()
            ]
        );

    match db.execute(del_stmt).await{
        Ok(exec_res) => {

            resp!{
                &[u8],
                &[],
                None, // metadata
                &format!("SUCCESS: record has been deleted with id: {}", line_id.to_owned()),
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
                "store_line.db.execute", // method
                Some(&zerlog_producer_actor)
            ).await;

            return Err(error_instance); // terminate the caller
        }
    }

}


#[put("/route/")]
pub(self) async fn update_line(
    req: HttpRequest,
    new_line: web::Json<UpdateLineRequest>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let new_line = new_line.to_owned();
    
    let json_points = serde_json::json!({
        "points": new_line.points
    });

    let geom_points: Vec<String> = new_line
        .points
        .iter()
        .map(|p| format!("{} {}", p.latitude, p.longitude))
        .collect();

    let route = format!("LINESTRING({})", geom_points.join(", "));

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let del_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::UPDATE_BY_ROUTE_ID, 
            [
                new_line.line_id.to_owned().into(),
                json_points.into(),
                route.into()
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
                "store_line.db.execute", // method
                Some(&zerlog_producer_actor)
            ).await;

            return Err(error_instance); // terminate the caller
        }
    }
}



#[post("/route/")]
pub(self) async fn store_line(
    req: HttpRequest,
    line_info: web::Json<LineRequest>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{

    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let line_info = line_info.to_owned();
    let imei = line_info.clone().imei;
    
    let json_points = serde_json::json!({
        "points": line_info.points
    });
    
    let geom_points: Vec<String> = line_info
        .points
        .iter()
        .map(|p| format!("{} {}", p.latitude, p.longitude))
        .collect();

    let route = format!("LINESTRING({})", geom_points.join(", "));

    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let insert_stmt = Statement::from_sql_and_values(db_backend, 
            consts::queries::INERT_LINE,
            [   
                imei.into(),
                json_points.into(),
                route.into(),
                line_info.tresh_hold.into(),
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
                 "store_line.db.execute", // method
                 Some(&zerlog_producer_actor)
             ).await;

             return Err(error_instance); // terminate the caller
         }
     }

}


// /route/?line_id=1
// /route/?imei=""
#[get("/route/")]
pub(self) async fn get_line(
    req: HttpRequest,
    line_q: web::Query<LineQuery>,
    app_state: web::Data<AppState>
) -> RustackiHttpResponse{


    let app_storage = app_state.app_storage.clone().unwrap();
    let db = app_storage.get_seaorm_pool().await.unwrap();
    let zerlog_producer_actor = app_state.actors.clone().unwrap().producer_actors.zerlog_actor;

    let line_q = line_q.0;
    let line_q_imei = line_q.imei;
    let get_g_id = line_q.line_id;


    /* -ˋˏ✄┈┈┈┈ saving using raw statement */
    let db_backend = db.get_database_backend(); // postgres in our case
    let query = if line_q_imei.is_some(){
        Statement::from_sql_and_values(db_backend, 
            consts::queries::GET_LINE_QUERY_BY_ROUTE_IMEI, 
            [
                line_q_imei.unwrap_or_default().into(),
            ]
        )
    } else if get_g_id.is_some(){
        Statement::from_sql_and_values(db_backend, 
            consts::queries::GET_LINE_QUERY_BY_ROUTE_ID, 
            [
                get_g_id.unwrap_or_default().into(),
            ]
        )
    } else{
        resp!{
            &[u8],
            &[],
            None, // metadata
            &format!("ERROR: there must be either imei or line id"),
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
                        id: d.try_get("route_id", "").unwrap(),
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
                 "get_line.db.execute", // method
                 Some(&zerlog_producer_actor)
             ).await;

             return Err(error_instance); // terminate the caller
         }
     }

}



pub mod exports{
    pub use super::store_line;
    pub use super::get_line;
    pub use super::delete_line;
    pub use super::update_line;
}