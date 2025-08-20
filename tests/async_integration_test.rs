// tests/async_integration_test.rs
pub mod schema;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_linker::relation;
use crate::schema::{users, posts, user_profiles, tags, post_tags};

#[derive(Clone, Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite", async = true)]
#[relation(model = "UserProfile", relation_type = "one_to_one", backend = "sqlite", async = true)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = user_profiles)]
pub struct UserProfile {
    pub id: i32,
    pub user_id: i32,
    pub bio: String,
}

#[derive(Clone, Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite", async = true)]
#[relation(
    model = "Tag",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "post_id",
    fk_child = "tag_id",
    backend = "sqlite",
    child_primary_key = "tag_id",
    async = true
)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

#[derive(Clone, Queryable, Identifiable, Insertable, Debug, PartialEq)]
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
    child_primary_key = "id",
    async = true
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

async fn setup_db() -> SyncConnectionWrapper<diesel::sqlite::SqliteConnection> {
    let mut conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();
    diesel::RunQueryDsl::execute(diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)"), &mut conn).unwrap();
    diesel::RunQueryDsl::execute(diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)"), &mut conn).unwrap();
    diesel::RunQueryDsl::execute(diesel::sql_query("CREATE TABLE user_profiles (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, bio TEXT NOT NULL)"), &mut conn).unwrap();
    diesel::RunQueryDsl::execute(diesel::sql_query("CREATE TABLE tags (tag_id INTEGER PRIMARY KEY, name TEXT NOT NULL)"), &mut conn).unwrap();
    diesel::RunQueryDsl::execute(diesel::sql_query("CREATE TABLE post_tags (id INTEGER PRIMARY KEY, post_id INTEGER NOT NULL, tag_id INTEGER NOT NULL)"), &mut conn).unwrap();
    SyncConnectionWrapper::new(conn)
}

#[tokio::test]
async fn test_one_to_many_get_async() {
    let mut conn = setup_db().await;

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).await.unwrap();

    let new_post = Post { id: 1, user_id: 1, title: "First post".to_string() };
    diesel::insert_into(posts::table).values(&new_post).execute(&mut conn).await.unwrap();

    let user = users::table.find(1).first::<User>(&mut conn).await.unwrap();
    let posts = user.get_posts(&mut conn).await.unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");

    let post = posts::table.find(1).first::<Post>(&mut conn).await.unwrap();
    let user_of_post = post.get_user(&mut conn).await.unwrap();
    assert_eq!(user_of_post.id, 1);
    assert_eq!(user_of_post.name, "Alice");
}

#[tokio::test]
async fn test_one_to_one_get_async() {
    use crate::schema::{users, user_profiles};
    let mut conn = setup_db().await;

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).await.unwrap();

    let new_profile = UserProfile { id: 1, user_id: 1, bio: "Alice's bio".to_string() };
    diesel::insert_into(user_profiles::table).values(&new_profile).execute(&mut conn).await.unwrap();

    let user = users::table.find(1).first::<User>(&mut conn).await.unwrap();
    let profile = user.get_userprofile(&mut conn).await.unwrap();

    assert_eq!(profile.bio, "Alice's bio");
}

#[tokio::test]
async fn test_many_to_many_get_async() {
    use crate::schema::{users, posts, tags, post_tags};
    let mut conn = setup_db().await;

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).await.unwrap();

    let new_post = Post { id: 1, user_id: 1, title: "First post".to_string() };
    diesel::insert_into(posts::table).values(&new_post).execute(&mut conn).await.unwrap();

    let new_tag = Tag { tag_id: 1, name: "rust".to_string() };
    diesel::insert_into(tags::table).values(&new_tag).execute(&mut conn).await.unwrap();

    let new_post_tag = PostTag { id: 1, post_id: 1, tag_id: 1 };
    diesel::insert_into(post_tags::table).values(&new_post_tag).execute(&mut conn).await.unwrap();

    let post = posts::table.find(1).first::<Post>(&mut conn).await.unwrap();
    let tags = post.get_tags(&mut conn).await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "rust");

    let tag = tags::table.find(1).first::<Tag>(&mut conn).await.unwrap();
    let posts = tag.get_posts(&mut conn).await.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");
}
