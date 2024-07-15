


pub const INSERT_QUERY: &str = r#"
    INSERT INTO hyper_geo_locations (imei, points, geom) VALUES ($1, $2, ST_GeomFromText($3, 4326))
"#;

pub const GET_QUERY_BY_GEO_ID: &str = r#"
    select id as geo_id, imei as device_imei, points as device_points from hyper_geo_locations where id = $1
"#;

pub const GET_QUERY_BY_GEO_IMEI: &str = r#"
    select id as geo_id, imei as device_imei, points as device_points from hyper_geo_locations where imei = $1
"#;

pub const DELETE_BY_GEO_ID: &str = r#"
    delete from hyper_geo_locations where id = $1
"#;

pub const UPDATE_BY_GEO_ID: &str = r#"
    update hyper_geo_locations set points = $2, geom = ST_GeomFromText($3, 4326) where id = $1
"#; 


pub const CHECK_QUERY: &str = r#"
    SELECT imei, id,  
    ST_AsText(geom) AS geom_text,
    ST_AsText(ST_SetSRID(ST_MakePoint($2, $3), 4326)) AS point_text,
    ST_Contains(geom, ST_SetSRID(ST_MakePoint($2, $3), 4326)) AS is_contains
    FROM hyper_geo_locations 
    WHERE imei = $1;
"#;

pub const INSERT_INTO_GEO_RESULT: &str = r#"
    insert into geo_result (geo_id, imei, geom_text, points_text, is_contains)
    values ($1, $2, $3, $4, $5)
"#;

// ---------------------------------------------------------------------------

pub const INERT_LINE: &str = r#"
    insert into route (imei, points, route, exit_tresh_hold) values 
    (
        $1, $2, ST_GeomFromText($3, 4326), $4
    )
"#;

pub const GET_LINE_QUERY_BY_ROUTE_ID: &str = r#"
    select id as route_id, imei as device_imei, points as device_points from route where id = $1
"#;

pub const GET_LINE_QUERY_BY_ROUTE_IMEI: &str = r#"
    select id as route_id, imei as device_imei, points as device_points from route where imei = $1
"#;


pub const DELETE_BY_ROUTE_ID: &str = r#"
    delete from route where id = $1
"#;

pub const UPDATE_BY_ROUTE_ID: &str = r#"
    update route set points = $2, route = ST_GeomFromText($3, 4326) where id = $1
"#; 


pub const CHECK_ROUTE_QUERY: &str = r#"
    SELECT imei, id, exit_tresh_hold, 
    ST_AsText(route) AS route_text,
    ST_AsText(ST_SetSRID(ST_MakePoint($2, $3), 4326)) AS point_text,
    ST_DWithin(route, ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography, exit_tresh_hold) AS is_contains
    FROM route 
    WHERE imei = $1;
"#;

pub const INSERT_INTO_ROUTE_RESULT: &str = r#"
    insert into route_result (route_id, imei, route_text, points_text, is_contains)
    values ($1, $2, $3, $4, $5)
"#;