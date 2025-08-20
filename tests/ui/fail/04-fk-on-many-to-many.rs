use diesel_linker::relation;

#[derive(Debug)]
struct Post;

#[derive(Debug)]
#[relation(
    relation_type = "many_to_many",
    model = "Post",
    fk = "user_id", // This is not used for a many_to_many relation
    join_table = "users_posts",
    fk_parent = "user_id",
    fk_child = "post_id",
    backend = "sqlite"
)]
struct User;

fn main() {}
