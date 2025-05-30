use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::convidados)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Convidados {
    pub id: i32,
    pub name: String
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::confirmacao)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Confirmacao {
    pub id_convidado: i32,
    pub estara_presente: bool 
}
