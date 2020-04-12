use actix_web::{web, App, HttpServer, HttpResponse, error, web::{Path, Json}};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

struct AppStateWithMutex {
    counter: Mutex<i32>,
    app_name: String,
    notes: Mutex<Vec<Note>>
}

#[derive(Serialize, Deserialize, Clone)]
struct Note {
    title: String,
    text: String,
}

async fn index(data: web::Data<AppStateWithMutex>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Request number; {}", counter)
}

async fn get_note(uid: Path<u32>, data: web::Data<AppStateWithMutex>) -> Result<Json<Note>, error::Error> {
    let notes = data.notes.lock().unwrap();
    let some_note = notes.get(uid.into_inner() as usize);
    match some_note {
        Some(x) => Ok(Json(x.clone())),
        None => Err(error::ErrorNotFound("This uid is not found")) 
    }
}

async fn get_notes(data: web::Data<AppStateWithMutex>) -> Result<Json<Vec<Note>>, error::Error> {
    let notes = data.notes.lock().unwrap();
    Ok(Json(notes.clone()))
}

async fn create_note(data: web::Data<AppStateWithMutex>, create_note_request: Json<Note>) -> Result<Json<Note>, error::Error> {
    let mut notes = data.notes.lock().unwrap();
    notes.push(create_note_request.clone());
    Ok(create_note_request)
}

async fn delete_note(uid: Path<u32>, data: web::Data<AppStateWithMutex>) -> Result<HttpResponse, error::Error> {
    let mut notes = data.notes.lock().unwrap();
    let i = uid.into_inner() as usize;
    match notes.get(i) {
        Some(_) => {
            notes.remove(i);
            Ok(HttpResponse::Ok().body("Deleting!"))
        },
        None => {
            Err(error::ErrorNotFound("Not found!"))
        }
    }
}

fn note_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/note")
            .route("/{uid}", web::get().to(get_note))
            .route("/{uid}", web::delete().to(delete_note))
            .route("", web::get().to(get_notes))
            .route("", web::post().to(create_note))
    );
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut stuff = Vec::new();
    stuff.push(Note{title: String::from("Test1"), text: String::from("This is a test note")});
    stuff.push(Note{title: String::from("Test2"), text: String::from("This is another test note")});

    let data = web::Data::new(AppStateWithMutex {
        counter: Mutex::new(0),
        app_name: String::from("Actix_Example"),
        notes: Mutex::new(stuff)
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
