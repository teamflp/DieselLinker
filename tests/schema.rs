use diesel::{table, joinable};

table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    posts (id) {
        id -> Integer,
        user_id -> Nullable<Integer>,
        title -> Text,
    }
}

joinable!(posts -> users (user_id));
