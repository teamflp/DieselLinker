use diesel_linker::relation;

#[derive(Debug)]
struct User;

#[derive(Debug)]
#[relation(
    relation_type = "many_to_one",
    model = "User",
    fk = "user_id",
    join_table = "users_posts", // This is not used for a many_to_one relation
    backend = "sqlite"
)]
struct Post;

fn main() {}
