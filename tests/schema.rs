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
        user_id -> Integer,
        title -> Text,
    }
}

joinable!(posts -> users (user_id));

table! {
    user_profiles (id) {
        id -> Integer,
        user_id -> Integer,
        bio -> Text,
    }
}

joinable!(user_profiles -> users (user_id));

table! {
    tags (tag_id) {
        tag_id -> Integer,
        name -> Text,
    }
}

table! {
    post_tags (id) {
        id -> Integer,
        post_id -> Integer,
        tag_id -> Integer,
    }
}

joinable!(post_tags -> posts (post_id));
joinable!(post_tags -> tags (tag_id));

table! {
    authors (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    books (id) {
        id -> Integer,
        author_id -> Integer,
        publisher_id -> Integer,
        title -> Text,
    }
}

table! {
    publishers (publisher_id) {
        publisher_id -> Integer,
        name -> Text,
    }
}

joinable!(books -> authors (author_id));
joinable!(books -> publishers (publisher_id));

use diesel::allow_tables_to_appear_in_same_query;
allow_tables_to_appear_in_same_query!(posts, post_tags, tags, authors, books, publishers);
