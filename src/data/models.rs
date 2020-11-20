use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;

use super::schema::notes;
use super::schema::notes::dsl::notes as notes_dsl;

#[derive(Serialize, Deserialize, Debug, Queryable, Insertable, Clone)]
#[table_name = "notes"]
pub struct Note {
    pub id: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct NewNote {
    pub title: String,
    pub text: String
}

impl Note {
    pub fn list(conn: &SqliteConnection) -> Vec<Self> {
        notes_dsl.load::<Note>(conn).expect("Error loading notes")
    }

    pub fn get(id: &str, conn: &SqliteConnection) -> Option<Self> {
        if let Ok(record) = notes_dsl.find(id).get_result::<Note>(conn) {
            Some(record)
        } else {
            None
        }
    }

    pub fn create(title: Option<&str>, text: Option<&str>, conn: &SqliteConnection) -> Option<Self> {
        let new_id = Uuid::new_v4().to_hyphenated().to_string();

        let new_note = Note{
            id: (&new_id).into(),
            title: title.map(Into::into),
            text: text.map(Into::into),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local()
        };

        diesel::insert_into(notes_dsl)
            .values(&new_note)
            .execute(conn)
            .expect("Error saving note");

        Self::get(&new_id, conn)
    }

    pub fn update(id: &str, utitle: Option<&str>, utext: Option<&str>, conn: &SqliteConnection) -> Option<Self> {
        use super::schema::notes::dsl::{title, text, updated_at};

        if utitle.is_none() && utext.is_none() {
            return Self::get(id, conn);
        }

        let updated_ts = chrono::Local::now().naive_local();

        diesel::update(notes_dsl.find(id))
            .set((
                title.eq(utitle),
                text.eq(utext),
                updated_at.eq(updated_ts)
            ))
            .execute(conn)
            .expect("Error updating note");

        Self::get(id, conn)
    }

    pub fn delete(id: &str, conn: &SqliteConnection) {
        if Self::get(id, conn).is_none() {
            println!("Note not found");
            return
        }

        diesel::delete(notes_dsl.find(id)).execute(conn).expect("Error deleting note");
    }
}