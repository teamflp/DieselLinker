pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;
use crate::schema::{users, posts, user_profiles, tags, post_tags};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
#[relation(model = "UserProfile", relation_type = "one_to_one", backend = "sqlite")]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = user_profiles)]
pub struct UserProfile {
    pub id: i32,
    pub user_id: i32,
    pub bio: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
#[relation(
    model = "Tag",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "post_id",
    fk_child = "tag_id",
    backend = "sqlite",
    child_primary_key = "tag_id"
)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = tags)]
#[diesel(primary_key(tag_id))]
#[relation(
    model = "Post",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "tag_id",
    fk_child = "post_id",
    backend = "sqlite",
    primary_key = "tag_id",
    child_primary_key = "id"
)]
pub struct Tag {
    pub tag_id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Post), belongs_to(Tag), table_name = post_tags)]
pub struct PostTag {
    pub id: i32,
    pub post_id: i32,
    pub tag_id: i32,
}

fn setup_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE user_profiles (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, bio TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE tags (tag_id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE post_tags (id INTEGER PRIMARY KEY, post_id INTEGER NOT NULL, tag_id INTEGER NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_one_to_many_get() {
    let mut conn = setup_db();

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let new_post = Post { id: 1, user_id: 1, title: "First post".to_string() };
    diesel::insert_into(posts::table).values(&new_post).execute(&mut conn).unwrap();

    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    let posts = user.get_posts(&mut conn).unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");

    let post = posts::table.find(1).first::<Post>(&mut conn).unwrap();
    let user_of_post = post.get_user(&mut conn).unwrap();
    assert_eq!(user_of_post.id, 1);
    assert_eq!(user_of_post.name, "Alice");
}

#[test]
fn test_one_to_one_get() {
    use crate::schema::{users, user_profiles};
    let mut conn = setup_db();

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let new_profile = UserProfile { id: 1, user_id: 1, bio: "Alice's bio".to_string() };
    diesel::insert_into(user_profiles::table).values(&new_profile).execute(&mut conn).unwrap();

    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    let profile = user.get_userprofile(&mut conn).unwrap();

    assert_eq!(profile.bio, "Alice's bio");
}

#[test]
fn test_many_to_many_get() {
    use crate::schema::{users, posts, tags, post_tags};
    let mut conn = setup_db();

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let new_post = Post { id: 1, user_id: 1, title: "First post".to_string() };
    diesel::insert_into(posts::table).values(&new_post).execute(&mut conn).unwrap();

    let new_tag = Tag { tag_id: 1, name: "rust".to_string() };
    diesel::insert_into(tags::table).values(&new_tag).execute(&mut conn).unwrap();

    let new_post_tag = PostTag { id: 1, post_id: 1, tag_id: 1 };
    diesel::insert_into(post_tags::table).values(&new_post_tag).execute(&mut conn).unwrap();

    let post = posts::table.find(1).first::<Post>(&mut conn).unwrap();
    let tags = post.get_tags(&mut conn).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "rust");

    let tag = tags::table.find(1).first::<Tag>(&mut conn).unwrap();
    let posts = tag.get_posts(&mut conn).unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");
}

// --- Test for custom method name ---

use crate::schema::{authors, books, publishers};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = authors)]
#[relation(model = "Book", relation_type = "one_to_many", backend = "sqlite", method_name = "fetch_books")]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq, Clone)]
#[diesel(table_name = publishers)]
#[diesel(primary_key(publisher_id))]
pub struct Publisher {
    pub publisher_id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq, Clone)]
#[diesel(belongs_to(Author), belongs_to(Publisher), table_name = books)]
#[relation(model = "Author", fk = "author_id", relation_type = "many_to_one", backend = "sqlite", method_name = "fetch_author")]
#[relation(
    model = "Publisher",
    fk = "publisher_id",
    relation_type = "many_to_one",
    backend = "sqlite",
    eager_loading = true,
    parent_primary_key = "publisher_id"
)]
pub struct Book {
    pub id: i32,
    pub author_id: i32,
    pub publisher_id: i32,
    pub title: String,
}

fn setup_custom_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    // The main setup_db() function already creates the tables we need for other tests.
    // We just need to add the new tables here.
    diesel::sql_query("CREATE TABLE authors (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE publishers (publisher_id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE books (id INTEGER PRIMARY KEY, author_id INTEGER NOT NULL, publisher_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_custom_method_name() {
    let mut conn = setup_custom_db();

    let new_publisher = Publisher { publisher_id: 1, name: "Penguin Books".to_string() };
    diesel::insert_into(publishers::table).values(&new_publisher).execute(&mut conn).unwrap();

    let new_author = Author { id: 1, name: "George Orwell".to_string() };
    diesel::insert_into(authors::table).values(&new_author).execute(&mut conn).unwrap();

    let new_book = Book { id: 1, author_id: 1, publisher_id: 1, title: "1984".to_string() };
    diesel::insert_into(books::table).values(&new_book).execute(&mut conn).unwrap();

    // Test the one-to-many relation with custom name
    let author = authors::table.find(1).first::<Author>(&mut conn).unwrap();
    let author_books = author.fetch_books(&mut conn).unwrap();

    assert_eq!(author_books.len(), 1);
    assert_eq!(author_books[0].title, "1984");

    // Test the many-to-one relation with custom name
    let book = books::table.find(1).first::<Book>(&mut conn).unwrap();
    let book_author = book.fetch_author(&mut conn).unwrap();
    assert_eq!(book_author.name, "George Orwell");
}

#[test]
fn test_eager_loading_with_custom_pk() {
    let mut conn = setup_custom_db();

    let new_publisher = Publisher { publisher_id: 1, name: "Penguin Books".to_string() };
    diesel::insert_into(publishers::table).values(&new_publisher).execute(&mut conn).unwrap();

    let new_author = Author { id: 1, name: "George Orwell".to_string() };
    diesel::insert_into(authors::table).values(&new_author).execute(&mut conn).unwrap();

    let books_to_insert = vec![
        Book { id: 1, author_id: 1, publisher_id: 1, title: "1984".to_string() },
        Book { id: 2, author_id: 1, publisher_id: 1, title: "Animal Farm".to_string() },
    ];
    diesel::insert_into(books::table).values(&books_to_insert).execute(&mut conn).unwrap();

    let books = books::table.load::<Book>(&mut conn).unwrap();
    let books_with_publishers = Book::load_with_publisher(books, &mut conn).unwrap();

    assert_eq!(books_with_publishers.len(), 2);
    for (_book, publisher) in books_with_publishers {
        assert_eq!(publisher.name, "Penguin Books");
    }
}