#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod database;
mod schema;

use database::models::{Counter, NewCounter};
use rocket::{fairing::AdHoc, serde::json::Json, *};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{
    gen::OpenApiGenerator,
    mount_endpoints_and_merged_docs,
    okapi::openapi3::OpenApi,
    openapi, openapi_get_routes_spec,
    rapidoc::*,
    request::{OpenApiFromRequest, RequestHeaderInput},
    swagger_ui::*,
};

use postgres;
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_sync_db_pools::{database, diesel::PgConnection};

// database connection made using the mayor crtaes in rust, my choice in order would be diesel, Sqlx, Postgres

#[database("test_db")]

struct Diesel_DbConn(PgConnection);

#[database("test_db")]

struct Postgres_DbConn(postgres::Client);

#[derive(Database)]
#[database("test_db")]
struct Sqlx_DbConn(sqlx::PgPool);

impl<'r> OpenApiFromRequest<'r> for Diesel_DbConn {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

/// # Home page
///
/// Get all records in database
#[openapi(tag = "Home")]
#[get("/")]
async fn all(conn: Diesel_DbConn) -> Json<Vec<Counter>> {
    let counters = conn
        .run(|c| database::actions::get_all_counters(&c))
        .await
        .unwrap();
    Json(counters)
}

#[openapi(tag = "Counters")]
#[get("/add/<name>/<number>")]
async fn add(name: String, number: u32, conn: Diesel_DbConn) -> String {
    let _counter = NewCounter {
        name,
        counter: number as i32,
    };
    let x = conn
        .run(|c| database::actions::add(&c, _counter).unwrap())
        .await;

    format!("Added {:?}", x)
}

#[openapi(tag = "Counters")]
#[get("/subtract/<name>/<number>")]
async fn subtract(name: String, number: u32, conn: Diesel_DbConn) -> String {
    let _counter = NewCounter {
        name,
        counter: -(number as i32),
    };
    let x = conn
        .run(|c| database::actions::subtract(&c, _counter))
        .await;

    format!("Subtracted: {:?}", x)
}

#[openapi(tag = "Counters")]
#[get("/status/<name>")]
async fn status(name: String, conn: Diesel_DbConn) -> String {
    let x = conn
        .run(|c| database::actions::get_counter_by_name(&c, name))
        .await;
    format!("Hello, {:?} ", x)
}

#[get("/sqlx")]
async fn sqlx_all(mut conn: Connection<Sqlx_DbConn>) -> String {
    // let x = &mut *conn;
    let x = database::actions::with_sqlx::all(&mut *conn).await;
    format!("with SQlx, {:?}", x)
}

#[get("/pg")]
async fn pg_all(conn: Postgres_DbConn) -> String {
    // let x = &mut *conn;
    let x = conn
        .run(|c| database::actions::with_postgres_crate::all(c))
        .await;
    format!("with postgres, {:?}", x)
}

#[get("/pg/add/<name>/<number>")]
async fn pg_add(name: String, number: u32, conn: Postgres_DbConn) -> String {
    let new_counter = NewCounter {
        name,
        counter: number as i32,
    };
    let x = conn
        .run(|c| database::actions::with_postgres_crate::add(c, new_counter))
        .await;
    format!("with postgres, {:?}", x)
}

#[get("/pg/subtract/<name>/<number>")]
async fn pg_subtract(name: String, number: u32, conn: Postgres_DbConn) -> String {
    let new_counter = NewCounter {
        name,
        counter: -(number as i32),
    };
    let x = conn
        .run(|c| database::actions::with_postgres_crate::subtract(c, new_counter))
        .await;
    format!("with postgres, {:?}", x)
}
async fn run_db_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    diesel_migrations::embed_migrations!();

    let conn = Diesel_DbConn::get_one(&rocket)
        .await
        .expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

#[launch]
async fn rocket() -> _ {
    let mut building_rocket = rocket::build()
        .attach(Diesel_DbConn::fairing())
        .attach(Postgres_DbConn::fairing())
        .attach(Sqlx_DbConn::init())
        .attach(AdHoc::on_ignite(
            "Initialise server schema",
            run_db_migrations,
        ))
        .mount("/", routes![sqlx_all, pg_all, pg_add, pg_subtract])
        .mount(
            "/docs/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/docs2/",
            make_rapidoc(&RapiDocConfig {
                title: Some("My special documentation | RapiDoc".to_owned()),
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    let custom_route_spec = (routes![], custom_openapi_spec());
    mount_endpoints_and_merged_docs! {
        building_rocket, "/".to_owned(), openapi_settings,
        "calcio" => custom_route_spec,
        "" =>  openapi_get_routes_spec!(openapi_settings: all,add, subtract, status  )
    };
    building_rocket
}

fn custom_openapi_spec() -> OpenApi {
    // use indexmap::indexmap;
    use rocket_okapi::okapi::openapi3::*;
    // use rocket_okapi::okapi::schemars::schema::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "The best counter API ever".to_owned(),
            description: Some("This is the best API every, please use me!".to_owned()),
            terms_of_service: Some(
                "https://github.com/GREsau/okapi/blob/master/LICENSE".to_owned(),
            ),
            contact: Some(Contact {
                name: Some("Dilec Padovani".to_owned()),
                url: Some("https://github.com/DILECPEDO".to_owned()),
                email: Some("test@test.com".to_owned()),
                ..Default::default()
            }),
            license: Some(License {
                name: "MIT".to_owned(),
                url: Some("https://github.com/GREsau/okapi/blob/master/LICENSE".to_owned()),
                ..Default::default()
            }),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        servers: vec![
            Server {
                url: "http://127.0.0.1:8000/".to_owned(),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: "https://example.com/".to_owned(),
                description: Some("Production Remote".to_owned()),
                ..Default::default()
            },
        ],
        // Add paths that do not exist in Rocket (or add extra into to existing paths)
        // paths: {
        //     indexmap! {
        //         "/home".to_owned() => PathItem{
        //         get: Some(
        //             Operation {
        //             tags: vec!["HomePage".to_owned()],
        //             summary: Some("This is my homepage".to_owned()),
        //             responses: Responses{
        //                 responses: indexmap!{
        //                 "200".to_owned() => RefOr::Object(
        //                     Response{
        //                     description: "Return the page, no error.".to_owned(),
        //                     content: indexmap!{
        //                         "text/html".to_owned() => MediaType{
        //                         schema: Some(SchemaObject{
        //                             instance_type: Some(SingleOrVec::Single(Box::new(
        //                                 InstanceType::String
        //                             ))),
        //                             ..Default::default()
        //                         }),
        //                         ..Default::default()
        //                         }
        //                     },
        //                     ..Default::default()
        //                     }
        //                 )
        //                 },
        //                 ..Default::default()
        //             },
        //             ..Default::default()
        //             }
        //         ),
        //         ..Default::default()
        //         }
        //     }
        // },
        ..Default::default()
    }
}
