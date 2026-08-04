use diesel_linker::relation;

#[relation(
    model = "Post",
    relation_type = "one_to_many",
    backend = "sqlite",
    error_type = "not valid rust"
)]
struct User;

fn main() {}
