


use crate::*;


// config is a mutable pointer to the web::ServiceConfig
// which makes to return nothing from each functions cause
// the state of the actual instance of web::ServiceConfig
// will be mutated in its scope


/*
     --------------------------------
    |     REGISTER EVENTS ROUTES
    | -------------------------------
    |
    |

*/
pub fn init(config: &mut web::ServiceConfig){

    config.service(apis::http::v1::events::geo::store_geo);
    config.service(apis::http::v1::events::geo::get_geo);
    config.service(apis::http::v1::events::geo::delete_geo);
    config.service(apis::http::v1::events::geo::update_geo);

    config.service(apis::http::v1::events::line::store_line);
    config.service(apis::http::v1::events::line::get_line);
    config.service(apis::http::v1::events::line::delete_line);
    config.service(apis::http::v1::events::line::update_line);

}