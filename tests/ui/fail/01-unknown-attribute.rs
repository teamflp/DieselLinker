use diesel_linker::relation;

#[relation(model = "Post", wrong_attr = "some_value")]
struct User;

fn main() {}
