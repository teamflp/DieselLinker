pub mod schema;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel_linker::relation;
use crate::schema::{users, posts, user_profiles, tags, post_tags};
use std::env;

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "postgres")]
#[relation(model = "UserProfile", relation_type = "one_to_one", backend = "postgres")]
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
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "postgres")]
#[relation(
    model = "Tag",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "post_id",
    fk_child = "tag_id",
    backend = "postgres",
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
    backend = "postgres",
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

fn setup_db() -> PgConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://test:test@localhost:5432/test".to_string());
    let mut conn = PgConnection::establish(&database_url).unwrap();
    diesel::sql_query("DROP TABLE IF EXISTS post_tags, tags, user_profiles, posts, users;").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE user_profiles (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL, bio TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE tags (tag_id SERIAL PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE post_tags (id SERIAL PRIMARY KEY, post_id INTEGER NOT NULL, tag_id INTEGER NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_one_to_many_get_pg() {
    let mut conn = setup_db();

    let new_user = diesel::insert_into(users::table).values(users::name.eq("Alice")).get_result::<User>(&mut conn).unwrap();
    let new_post = diesel::insert_into(posts::table)
        .values((posts::user_id.eq(new_user.id), posts::title.eq("First post")))
        .get_result::<Post>(&mut conn)
        .unwrap();

    let user = users::table.find(new_user.id).first::<User>(&mut conn).unwrap();
    let posts = user.get_posts(&mut conn).unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");

    let post = posts::table.find(new_post.id).first::<Post>(&mut conn).unwrap();
    let user_of_post = post.get_user(&mut conn).unwrap();
    assert_eq!(user_of_post.id, new_user.id);
    assert_eq!(user_of_post.name, "Alice");
}

#[test]
fn test_one_to_one_get_pg() {
    use crate::schema::{users, user_profiles};
    let mut conn = setup_db();

    let new_user = diesel::insert_into(users::table).values(users::name.eq("Alice")).get_result::<User>(&mut conn).unwrap();
    diesel::insert_into(user_profiles::table)
        .values((user_profiles::user_id.eq(new_user.id), user_profiles::bio.eq("Alice's bio")))
        .execute(&mut conn)
        .unwrap();

    let user = users::table.find(new_user.id).first::<User>(&mut conn).unwrap();
    let profile = user.get_userprofile(&mut conn).unwrap();

    assert_eq!(profile.bio, "Alice's bio");
}

#[test]
fn test_many_to_many_get_pg() {
    use crate::schema::{users, posts, tags, post_tags};
    let mut conn = setup_db();

    let new_user = diesel::insert_into(users::table).values(users::name.eq("Alice")).get_result::<User>(&mut conn).unwrap();
    let new_post = diesel::insert_into(posts::table)
        .values((posts::user_id.eq(new_user.id), posts::title.eq("First post")))
        .get_result::<Post>(&mut conn)
        .unwrap();
    let new_tag = diesel::insert_into(tags::table).values(tags::name.eq("rust")).get_result::<Tag>(&mut conn).unwrap();
    diesel::insert_into(post_tags::table)
        .values((post_tags::post_id.eq(new_post.id), post_tags::tag_id.eq(new_tag.tag_id)))
        .execute(&mut conn)
        .unwrap();

    let post = posts::table.find(new_post.id).first::<Post>(&mut conn).unwrap();
    let tags = post.get_tags(&mut conn).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "rust");

    let tag = tags::table.find(new_tag.tag_id).first::<Tag>(&mut conn).unwrap();
    let posts = tag.get_posts(&mut conn).unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");
}
