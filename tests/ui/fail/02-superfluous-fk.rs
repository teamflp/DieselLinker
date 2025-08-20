use diesel_linker::relation;

#[derive(Debug)]
struct User;

#[derive(Debug)]
#[relation(
    relation_type = "one_to_one",
    model = "User",
    fk = "user_id", // This is not used for a one_to_one relation
    backend = "sqlite"
)]
struct UserProfile;

fn main() {}
