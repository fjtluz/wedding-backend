mod schema;
mod models;

use diesel::{dsl::sql, prelude::*, sql_types::{Bool, Text}};
use dotenvy::dotenv;
use models::{Confirmacao, Convidados};
use rocket::{launch, post, response, routes, serde::{self, json::Json}, Responder};
use std::env;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    rocket::build().mount("/", routes![handle_confirmation])
}

#[derive(serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConfirmationInput {
    guest_name: String,
    will_attend: bool
}

#[derive(Responder)]
enum Response {
    #[response(status = 204)]
    Confirmed(String),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 500, content_type = "json")]
    Unexpected(String)
}

#[post("/confirmation", data = "<confirmation>")]
fn handle_confirmation(confirmation: Json<ConfirmationInput>) -> Response {
    if let Some(invitation) = is_guest_invited(&confirmation.guest_name) {
        return match persist_confirmation(invitation.id, confirmation.will_attend) {
            Ok(_) => Response::Confirmed(String::new()),
            Err(e) => Response::Unexpected(format!("Um erro inesperado ocorreu: {}", e)) 
        }
    } 

    Response::NotFound(String::from("{ \"response\": \"Não foi encontrado nenhum convidado com este nome!\"  }")) 
}

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set in .env file!");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error while connecting to {}", database_url))
}

fn is_guest_invited(guest_name: &String) -> Option<Convidados> {
    use self::schema::convidados::dsl::*;

    let connection = &mut establish_connection();

    let guest_name_upper = guest_name.to_uppercase();

    return convidados
        .filter(sql::<Bool>("UPPER(name) = ").bind::<Text, _>(guest_name_upper))
        .select(Convidados::as_select())
        .first(connection)
        .ok();
}

fn persist_confirmation(guest_id: i32, will_attend: bool) -> Result<usize, diesel::result::Error> {
    use self::schema::confirmacao;

    let connection = &mut establish_connection();

    let confirmation = Confirmacao {
        id_convidado: guest_id,
        estara_presente: will_attend
    };

    diesel::insert_into(confirmacao::table)
        .values(&confirmation)
        .execute(connection)
}
