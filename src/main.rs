#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{web, App, HttpServer, HttpResponse, error, web::{Path, Json}};
use std::sync::Mutex;

mod data;
use data::models::NewNote;
use data::models::Note;
use data::establish_connection;

use diesel::prelude::SqliteConnection;

struct AppStateWithMutex {
    counter: Mutex<i32>,
    conn: Mutex<SqliteConnection>
}

async fn index(data: web::Data<AppStateWithMutex>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Request number; {}", counter)
}

async fn get_note(uid: Path<String>, data: web::Data<AppStateWithMutex>) -> Result<Json<Note>, error::Error> {
    let conn = data.conn.lock().unwrap();
    let some_note = Note::get(uid.as_ref(), &conn);
    match some_note {
        Some(x) => Ok(Json(x.clone())),
        None => Err(error::ErrorNotFound("This uid is not found"))
    }
}

async fn get_notes(data: web::Data<AppStateWithMutex>) -> Result<Json<Vec<Note>>, error::Error> {
    let conn = data.conn.lock().unwrap();
    let notes = Note::list(&conn);
    Ok(Json(notes.clone()))
}

async fn create_note(data: web::Data<AppStateWithMutex>, create_note_request: Json<NewNote>) -> Result<Json<Note>, error::Error> {
    let conn = data.conn.lock().unwrap();
    let title = &create_note_request.title;
    let text = &create_note_request.text;
    let created_note = Note::create(Some(&title), Some(&text), &conn);
    match created_note {
        Some(x) => Ok(Json(x.clone())),
        None => Err(error::ErrorInternalServerError("Unable to create note"))
    }
}

async fn update_note(uid: Path<String>, data: web::Data<AppStateWithMutex>, update_note_request: Json<NewNote>) -> Result<Json<Note>, error::Error> {
    let conn = data.conn.lock().unwrap();
    let title = &update_note_request.title;
    let text = &update_note_request.text;

    let updated_note = Note::update(uid.as_ref(),Some(&title), Some(&text), &conn);
    match updated_note {
        Some(x) => Ok(Json(x.clone())),
        None => Err(error::ErrorNotFound("Note not found"))
    }
}

async fn delete_note(uid: Path<String>, data: web::Data<AppStateWithMutex>) -> Result<HttpResponse, error::Error> {
    let conn = data.conn.lock().unwrap();
    Note::delete(uid.as_ref(), &conn);
    Ok(HttpResponse::Ok().body("Deleting!"))
}

fn note_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/note")
            .route("/{uid}", web::get().to(get_note))
            .route("/{uid}", web::put().to(update_note))
            .route("/{uid}", web::delete().to(delete_note))
            .route("", web::get().to(get_notes))
            .route("", web::post().to(create_note))
    );
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let conn = establish_connection();
    let data = web::Data::new(AppStateWithMutex {
        counter: Mutex::new(0),
        conn: Mutex::new(conn)
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(note_config)
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .workers(3) //Number of thread workers to spin up. Default is # of logical cores.
    .run()
    .await
}
