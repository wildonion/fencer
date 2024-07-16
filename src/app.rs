
/*

Coded by


 █     █░ ██▓ ██▓    ▓█████▄  ▒█████   ███▄    █  ██▓ ▒█████   ███▄    █ 
▓█░ █ ░█░▓██▒▓██▒    ▒██▀ ██▌▒██▒  ██▒ ██ ▀█   █ ▓██▒▒██▒  ██▒ ██ ▀█   █ 
▒█░ █ ░█ ▒██▒▒██░    ░██   █▌▒██░  ██▒▓██  ▀█ ██▒▒██▒▒██░  ██▒▓██  ▀█ ██▒
░█░ █ ░█ ░██░▒██░    ░▓█▄   ▌▒██   ██░▓██▒  ▐▌██▒░██░▒██   ██░▓██▒  ▐▌██▒
░░██▒██▓ ░██░░██████▒░▒████▓ ░ ████▓▒░▒██░   ▓██░░██░░ ████▓▒░▒██░   ▓██░
░ ▓░▒ ▒  ░▓  ░ ▒░▓  ░ ▒▒▓  ▒ ░ ▒░▒░▒░ ░ ▒░   ▒ ▒ ░▓  ░ ▒░▒░▒░ ░ ▒░   ▒ ▒ 
  ▒ ░ ░   ▒ ░░ ░ ▒  ░ ░ ▒  ▒   ░ ▒ ▒░ ░ ░░   ░ ▒░ ▒ ░  ░ ▒ ▒░ ░ ░░   ░ ▒░
  ░   ░   ▒ ░  ░ ░    ░ ░  ░ ░ ░ ░ ▒     ░   ░ ░  ▒ ░░ ░ ░ ▒     ░   ░ ░ 
    ░     ░      ░  ░   ░        ░ ░           ░  ░      ░ ░           ░ 
                      ░                                                  


    -----------------------------------------------------------------
   |          NOTE ON CODE ORDER EXECUTION OF ASYNC METHODS
   |-----------------------------------------------------------------
   | in rust the order of execution is not async by default but rather it's thread safe 
   | and without having race conditions due to its rules of mutable and immutable pointers 
   | of types although if there might be async methods but it's not expected that they must 
   | be executed asyncly, the early one gets executed first and then the second one goes, 
   | an example of that would be calling async_method_one() method with async operations 
   | inside, and other_async_method_two() method, both of them are async however, but the 
   | code waits till all the async operations inside the first one get executed then run the 
   | second one, this gets criticized if we have some delay and sleep methods inside the first 
   | one which gets us into trouble with the whole process of code order execution if we don't 
   | want to have disruption in their execution, though in some cases it's needed to have this 
   | logic but in here it would be a bad idea, the solution to this is running both of them 
   | asyncly in their own seprate threadpool which can be done by putting each of them inside 
   | tokio::spawn() in this case there would be no disruption in their order execution at all 
   | and we'd have a fully async execution of methods in the background.
   | to catch any result data inside the tokio::spawn() we would have to use mpsc channel to
   | send the data to the channel inside the tokio::spawn() and receive it outside of tokio
   | scope and do the rest of the logics with that.
   | notice of putting .await after async task call, it consumes the future objcet by pinning
   | it into the ram for future solvation also it suspend the function execution until the future
   | gets compeleted allows other parts of the app get executed without having any disruption,
   | later once the future has completed waker sends the value back to the caller to update the 
   | state and its value, this behaviour is called none blocking completion but based on the 
   | beginning notes the order of async task execution is not async by itself and if we have 
   | async_task1 first followed by async_task2 the order of executing them is regular and it's 
   | not pure async like goroutines in Go or event loop in NodeJS, to achive this we should spawn 
   | them insie tokio::spawn which runs all the futures in the background like what goroutines 
   | and NodeJS event loop do
   |
   | conclusion: 
   | use tokio::spawn() to execute any async task in the background without having
   | any disruption in other order execution of async methods.
   | 


    -----------------------------------------------------------------
   |                  ACTIX RUNTIME VS TOKIO RUNTIME
   |-----------------------------------------------------------------
   | tokio runtime is an async task scheduler mainly schedule the time of execution of async tasks in the 
   | whole app, to use actix actors, http and ws we need to add #[actix_web::main] on top of main method 
   | but this doesn't mean we can't use the tokio::spawn() to execute task in the background putting #[tokio::main] 
   | doesn't allow us to run the actix tasks in actix runtime itself, note that we should avoid starting 
   | any actor in the background inside tokio::spawn otherwise we'll face the error of `spawn_local` called 
   | from outside of a `task::LocalSet` which says that we can't start an actor inside the tokio runtime 
   | instead we should use the actix runtime and start the actor in the context of actix runtime itself 
   | where there is no #[tokio::main] on top of main method, good to know that #[tokio::main] simplifies 
   | the sentup and execution of async tasks within the tokio runtime on the other hans tokio::spawn is 
   | used for manually spawning, managing and scheduling async task in the background you need to handle 
   | task lifecycles, cancellation, and coordination between tasks explicitly.
   |

*/

use crate::actors::consumers::location::ConsumeNotif;
use crate::actors::consumers::route::ConsumeNotif as RouteConsumeNotif;
use crate::actors::producers::location::ProduceNotif;
use crate::consts::APP_NAME;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use actix_web::middleware::Logger;
use actors::consumers::route::RouteLocationConsumerActor;
use consts::{MAILBOX_CHANNEL_ERROR_CODE, SERVERS};
use clap::Parser;
use dotenv::dotenv;
use lapin::options::{BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use std::io::BufWriter;
use std::str::FromStr;
use std::{fs::OpenOptions, io::BufReader};
use rand::Rng;
use rand::random;
use sha2::{Digest, Sha256};
use redis::Client as RedisClient;
use redis::AsyncCommands; // this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use redis::Commands;
use redis_async::client::{self, PubsubConnection, ConnectionBuilder};
use redis::RedisError;
use hyper::StatusCode;
use uuid::Uuid;
use log::{info, error};
use actix_redis::{Command, RedisActor, resp_array, RespValue};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix_cors::Cors;
use actix_web::{App, web, cookie::{self, Cookie, time::Duration, time::OffsetDateTime}, 
                web::Data, http::header, HttpRequest, middleware,
                HttpServer, Responder, HttpResponse, get, post, ResponseError};
use actix_multipart::Multipart;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use std::fmt::Write;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use tokio::time::Duration as TokioDuration;
use chrono::Utc;
use std::time::Instant;
use std::collections::HashSet;
use rand::rngs::ThreadRng;
use futures::StreamExt; /* is required to call the next() method on the streams */
use once_cell::sync::Lazy;
use std::rc::Weak;
use tokio::sync::RwLock;
use migration::{Migrator, MigratorTrait};


// all macros in other crates will be loaded in here 
// so accessing them from other crates and modules is
// like use crate::macro_name;

mod actors;
mod plugins;
mod s3;
mod config;
mod consts;
mod lockers;
mod error;
mod server;
mod types;
mod appstate;
mod services;
mod requests;
mod apis;
mod extractors;
mod cli;
mod tests;
mod models;
mod entities;



/* ******************************* IMPORTANT *******************************
    can't start tokio stuffs or actix stuffs like actors inside the context
    of actix or tokio runtime, so we can't have a pure TCP server with 
    actix_web::main HTTP server with tokio::main. 
 ************************************************************************* */
#[actix_web::main]
async fn main() -> std::io::Result<()>{

    /* -ˋˏ✄┈┈┈┈ logging
        >_
    */
    let args = cli::ServerKind::parse();
    dotenv::dotenv().expect("expected .env file be there!");
    env::set_var("RUST_LOG", "trace");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    // env::set_var("RUST_LOG", "actix_web=debug");


    /* -ˋˏ✄┈┈┈┈ initializing appstate actor workers
        >_ run actor workers, app_state contains the whole app data 
        which will be used globally during the execution of the app
    */
    let app_state = appstate::AppState::init().await;


    /* -ˋˏ✄┈┈┈┈ migrating on startup
        >_ ORM checks on its own that the db is up to create the pool connection
        it won't start the app if the db is off, makes sure you've started
        the pg db server
    */
    let connection = sea_orm::Database::connect(
        &app_state.config.as_ref().unwrap().vars.DATABASE_URL
    ).await.unwrap();
    let fresh = args.fresh;
    if fresh{
        Migrator::fresh(&connection).await.unwrap();
    } else{
        Migrator::up(&connection, None).await.unwrap(); // executing database tasks like creating tables on startup
    }
            
    /* -ˋˏ✄┈┈┈┈ bootstrapping http server
        >_ 
    */
    info!("➔ 🚀 {} HTTP+WebSocket socket server has launched from [{}:{}] at {}", 
        APP_NAME, app_state.config.as_ref().unwrap().vars.HOST, 
        app_state.config.as_ref().unwrap().vars.HTTP_PORT.parse::<u16>().unwrap(), 
        chrono::Local::now().naive_local()); 

    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    // start consuming in the background for geofence checker
    tokio::spawn(
        {
            let cloned_app_state = app_state.clone();
            let cloned_notif = ConsumeNotif{ 
                queue: String::from("RustGeofence"), 
                exchange_name: String::from("Events:LocationEvent"), 
                routing_key: String::from(""), 
                tag: String::from("geo_consume_tag0"), 
                redis_cache_exp: 500 
            };

            let zerlog_producer_actor = cloned_app_state.clone().actors.unwrap().producer_actors.zerlog_actor;
            async move{
                // consuming notif by sending the ConsumeNotif message to 
                // the consumer actor,
                match cloned_app_state.clone().actors.as_ref().unwrap()
                        .consumer_actors.location_actor.send(cloned_notif).await
                    {
                        Ok(ok) => {ok},
                        Err(e) => {
                            let source = &e.source().unwrap().to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                            let err_instance = crate::error::RustackiErrorResponse::new(
                                *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                &String::from("main.consumer_actors.location_actor.send"), // current method name
                                Some(&zerlog_producer_actor)
                            ).await;
                            return;
                        }
                    }

            }
        }
    );
    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    
    let app_storage = app_state.clone().app_storage;
    let actors = app_state.clone().actors.unwrap();
    let zerlog_producer_actor = actors.producer_actors.zerlog_actor;
    let notif_producer_actor = actors.producer_actors.location_actor;
    let route_checker_actor = RouteLocationConsumerActor::new(
        app_storage.clone(),
        notif_producer_actor,
        zerlog_producer_actor,
    ).start();
        
    // start consuming location event in the background for route checker
    tokio::spawn(
        {
            let cloned_app_state = app_state.clone();
            let cloned_notif = RouteConsumeNotif{ 
                queue: String::from("RustGeofenceRoute"), 
                exchange_name: String::from("Events:LocationEvent"), 
                routing_key: String::from(""), 
                tag: String::from("geo_consume_tag1"), 
                redis_cache_exp: 500 
            };

            let zerlog_producer_actor = cloned_app_state.clone().actors.unwrap().producer_actors.zerlog_actor;
            async move{
                // consuming notif by sending the ConsumeNotif message to 
                // the consumer actor,
                match route_checker_actor.send(cloned_notif).await
                    {
                        Ok(ok) => {ok},
                        Err(e) => {
                            let source = &e.source().unwrap().to_string(); // we know every goddamn type implements Error trait, we've used it here which allows use to call the source method on the object
                            let err_instance = crate::error::RustackiErrorResponse::new(
                                *MAILBOX_CHANNEL_ERROR_CODE, // error hex (u16) code
                                source.as_bytes().to_vec(), // text of error source in form of utf8 bytes
                                crate::error::ErrorKind::Actor(crate::error::ActixMailBoxError::Mailbox(e)), // the actual source of the error caused at runtime
                                &String::from("main.consumer_actors.route.send"), // current method name
                                Some(&zerlog_producer_actor)
                            ).await;
                            return;
                        }
                    }

            }
        }
    );
    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    ///// ------------------------------------------------------------------------------------
    
    tokio::spawn(
        {
            let cloned_app_state = app_state.clone();
            let storage = cloned_app_state.clone();
            let rmq_pool = storage.clone().app_storage.clone().unwrap().get_lapin_pool().await.unwrap();
            let conn = rmq_pool.get().await.unwrap();

            async move{
                let chan = conn.create_channel().await.unwrap();
            let q = chan
                .queue_declare(
                    &String::from("LocalGeoQ"),
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await.unwrap();
            chan
                .queue_bind(q.name().as_str(), &String::from("GeofenceExchange"), "", 
                    QueueBindOptions::default(), FieldTable::default()
                )
                .await;
        
            let mut consumer = chan
                .basic_consume(
                    // the queue that is bounded to the exchange to receive messages based on the routing key
                    // since the queue is already bounded to the exchange and its routing key it only receives 
                    // messages from the exchange that matches and follows the passed in routing pattern like:
                    // message routing key "orders.processed" might match a binding with routing key "orders.#
                    // if none the messages follow the pattern then the queue will receive no message from the 
                    // exchange based on that pattern!
                    "LocalGeoQ", 
                    "l_g0", // custom consumer name
                    BasicConsumeOptions::default(), 
                    FieldTable::default()
                )
                .await.unwrap();

                while let Some(delivery) = consumer.next().await{
                    match delivery{
                        Ok(delv) => {

                            let buffer = delv.data;
                            let data = std::str::from_utf8(&buffer).unwrap();
                            log::info!("delivery data from LocalGeoQ:::: {}", data);

                        },
                        Err(e) => {

                        }
                    }
                }
            }
        }
    );

    bootsteap_http!{
        app_state.clone(),
    }
        

}
