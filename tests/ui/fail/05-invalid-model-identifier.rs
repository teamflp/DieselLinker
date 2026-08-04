use diesel_linker::relation;

#[relation(model = "123 Not An Ident", relation_type = "one_to_many", backend = "sqlite")]
struct User;

fn main() {}
