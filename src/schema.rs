use diesel::table;

table! {
    casamento.convidados (id) {
        id -> Int4,
        name -> Varchar
    }
}

table! {
    casamento.confirmacao (id_convidado) {
        id_convidado -> Int4,
        estara_presente -> Bool
    }
}
