mod schema;
mod models;

use diesel::{dsl::sql, prelude::*, sql_types::{Bool, Text}};
use dotenvy::dotenv;
use models::{Confirmacao, Convidados};
use rocket::{fairing::{Fairing, Info, Kind}, http::Header, launch, options, post, request, routes, serde::{self, json::Json}, Request, Responder};
use std::env;

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin Resource Sharing (CORS)",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut rocket::Response<'r>) {
        let is_preflight = request.method() == rocket::http::Method::Options;

        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methos", "POST, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if is_preflight {
            response.set_status(rocket::http::Status::Ok);
        }
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();


    rocket::build()
        .mount("/", routes![handle_confirmation, confirmation_options])
        .attach(Cors)
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

#[options("/confirmation")]
fn confirmation_options() -> &'static str {
    ""
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

    let entry_exists = confirmacao::dsl::confirmacao.find(guest_id).first::<models::Confirmacao>(connection);
    match entry_exists {
        Ok(_) => diesel::update(confirmacao::dsl::confirmacao.find(guest_id))
            .set(self::schema::confirmacao::dsl::estara_presente.eq(will_attend))
            .execute(connection),
        Err(_) => diesel::insert_into(confirmacao::table)
            .values(&confirmation)
            .execute(connection)
    }
}
