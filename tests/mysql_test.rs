pub mod schema;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::sql_types::{Unsigned, BigInt};
use diesel_linker::relation;
use crate::schema::{users, posts, user_profiles, tags, post_tags};
use std::env;

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq, Clone)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "mysql", eager_loading = true)]
#[relation(model = "UserProfile", relation_type = "one_to_one", backend = "mysql", eager_loading = true)]
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

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq, Clone)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "mysql", eager_loading = true)]
#[relation(
    model = "Tag",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "post_id",
    fk_child = "tag_id",
    backend = "mysql",
    child_primary_key = "tag_id",
    eager_loading = true
)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq, Clone)]
#[diesel(table_name = tags)]
#[diesel(primary_key(tag_id))]
#[relation(
    model = "Post",
    relation_type = "many_to_many",
    join_table = "post_tags",
    fk_parent = "tag_id",
    fk_child = "post_id",
    backend = "mysql",
    primary_key = "tag_id",
    child_primary_key = "id",
    eager_loading = true
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

fn setup_db() -> MysqlConnection {
    let database_url = env::var("MYSQL_URL").unwrap_or_else(|_| "mysql://root:password@localhost:3306/diesel_linker_test".to_string());
    let mut conn = MysqlConnection::establish(&database_url).unwrap();
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 0;").execute(&mut conn).unwrap();
    diesel::sql_query("DROP TABLE IF EXISTS post_tags, tags, user_profiles, posts, users;").execute(&mut conn).unwrap();
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 1;").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTO_INCREMENT, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY AUTO_INCREMENT, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE user_profiles (id INTEGER PRIMARY KEY AUTO_INCREMENT, user_id INTEGER NOT NULL, bio TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE tags (tag_id INTEGER PRIMARY KEY AUTO_INCREMENT, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE post_tags (id INTEGER PRIMARY KEY AUTO_INCREMENT, post_id INTEGER NOT NULL, tag_id INTEGER NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_one_to_many_get_mysql() {
    let mut conn = setup_db();

    diesel::insert_into(users::table).values(users::name.eq("Alice")).execute(&mut conn).unwrap();
    let new_user_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_user = users::table.find(new_user_id).first::<User>(&mut conn).unwrap();

    diesel::insert_into(posts::table)
        .values((posts::user_id.eq(new_user.id), posts::title.eq("First post")))
        .execute(&mut conn)
        .unwrap();
    let new_post_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_post = posts::table.find(new_post_id).first::<Post>(&mut conn).unwrap();

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
fn test_one_to_one_get_mysql() {
    use crate::schema::{users, user_profiles};
    let mut conn = setup_db();

    diesel::insert_into(users::table).values(users::name.eq("Alice")).execute(&mut conn).unwrap();
    let new_user_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_user = users::table.find(new_user_id).first::<User>(&mut conn).unwrap();

    diesel::insert_into(user_profiles::table)
        .values((user_profiles::user_id.eq(new_user.id), user_profiles::bio.eq("Alice's bio")))
        .execute(&mut conn)
        .unwrap();

    let user = users::table.find(new_user.id).first::<User>(&mut conn).unwrap();
    let profile = user.get_userprofile(&mut conn).unwrap();

    assert_eq!(profile.bio, "Alice's bio");
}

#[test]
fn test_many_to_many_get_mysql() {
    use crate::schema::{users, posts, tags, post_tags};
    let mut conn = setup_db();

    diesel::insert_into(users::table).values(users::name.eq("Alice")).execute(&mut conn).unwrap();
    let new_user_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_user = users::table.find(new_user_id).first::<User>(&mut conn).unwrap();

    diesel::insert_into(posts::table)
        .values((posts::user_id.eq(new_user.id), posts::title.eq("First post")))
        .execute(&mut conn)
        .unwrap();
    let new_post_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_post = posts::table.find(new_post_id).first::<Post>(&mut conn).unwrap();

    diesel::insert_into(tags::table).values(tags::name.eq("rust")).execute(&mut conn).unwrap();
    let new_tag_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    let new_tag = tags::table.find(new_tag_id).first::<Tag>(&mut conn).unwrap();

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

#[test]
fn test_one_to_many_eager_loading_mysql() {
    let mut conn = setup_db();

    // --- User 1 with 2 posts ---
    diesel::insert_into(users::table).values(users::name.eq("User 1")).execute(&mut conn).unwrap();
    let user1_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(posts::table)
        .values(&[
            (posts::user_id.eq(user1_id), posts::title.eq("Post 1.1")),
            (posts::user_id.eq(user1_id), posts::title.eq("Post 1.2")),
        ])
        .execute(&mut conn)
        .unwrap();

    // --- User 2 with 1 post ---
    diesel::insert_into(users::table).values(users::name.eq("User 2")).execute(&mut conn).unwrap();
    let user2_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(posts::table)
        .values((posts::user_id.eq(user2_id), posts::title.eq("Post 2.1")))
        .execute(&mut conn)
        .unwrap();

    // --- User 3 with 0 posts ---
    diesel::insert_into(users::table).values(users::name.eq("User 3")).execute(&mut conn).unwrap();
    let _user3_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    let all_users = users::table.order(users::id.asc()).load::<User>(&mut conn).unwrap();
    assert_eq!(all_users.len(), 3);

    let users_with_posts = User::load_with_posts(all_users, &mut conn).unwrap();

    assert_eq!(users_with_posts.len(), 3);

    // Check User 1
    assert_eq!(users_with_posts[0].0.name, "User 1");
    assert_eq!(users_with_posts[0].1.len(), 2);
    assert_eq!(users_with_posts[0].1[0].title, "Post 1.1");
    assert_eq!(users_with_posts[0].1[1].title, "Post 1.2");

    // Check User 2
    assert_eq!(users_with_posts[1].0.name, "User 2");
    assert_eq!(users_with_posts[1].1.len(), 1);
    assert_eq!(users_with_posts[1].1[0].title, "Post 2.1");

    // Check User 3
    assert_eq!(users_with_posts[2].0.name, "User 3");
    assert_eq!(users_with_posts[2].1.len(), 0);
}

#[test]
fn test_one_to_one_eager_loading_mysql() {
    let mut conn = setup_db();
    use crate::schema::{users, user_profiles};

    // --- User 1 with profile ---
    diesel::insert_into(users::table).values(users::name.eq("User 1")).execute(&mut conn).unwrap();
    let user1_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(user_profiles::table)
        .values((user_profiles::user_id.eq(user1_id), user_profiles::bio.eq("Bio 1")))
        .execute(&mut conn)
        .unwrap();

    // --- User 2 without profile ---
    diesel::insert_into(users::table).values(users::name.eq("User 2")).execute(&mut conn).unwrap();
    let _user2_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    let all_users = users::table.order(users::id.asc()).load::<User>(&mut conn).unwrap();
    assert_eq!(all_users.len(), 2);

    let users_with_profiles = User::load_with_userprofile(all_users, &mut conn).unwrap();

    assert_eq!(users_with_profiles.len(), 2);

    // Check User 1
    assert_eq!(users_with_profiles[0].0.name, "User 1");
    assert_eq!(users_with_profiles[0].1.len(), 1);
    assert_eq!(users_with_profiles[0].1[0].bio, "Bio 1");

    // Check User 2
    assert_eq!(users_with_profiles[1].0.name, "User 2");
    assert_eq!(users_with_profiles[1].1.len(), 0);
}

#[test]
fn test_many_to_one_eager_loading_mysql() {
    let mut conn = setup_db();

    // --- User 1 ---
    diesel::insert_into(users::table).values(users::name.eq("User 1")).execute(&mut conn).unwrap();
    let user1_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    // --- User 2 ---
    diesel::insert_into(users::table).values(users::name.eq("User 2")).execute(&mut conn).unwrap();
    let user2_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    // --- Posts for users ---
    diesel::insert_into(posts::table)
        .values(&[
            (posts::user_id.eq(user1_id), posts::title.eq("Post 1.1")),
            (posts::user_id.eq(user2_id), posts::title.eq("Post 2.1")),
            (posts::user_id.eq(user1_id), posts::title.eq("Post 1.2")),
        ])
        .execute(&mut conn)
        .unwrap();

    let all_posts = posts::table.order(posts::id.asc()).load::<Post>(&mut conn).unwrap();
    assert_eq!(all_posts.len(), 3);

    let posts_with_users = Post::load_with_user(all_posts, &mut conn).unwrap();
    assert_eq!(posts_with_users.len(), 3);

    // Post 1.1 -> User 1
    assert_eq!(posts_with_users[0].0.title, "Post 1.1");
    assert_eq!(posts_with_users[0].1.name, "User 1");

    // Post 2.1 -> User 2
    assert_eq!(posts_with_users[1].0.title, "Post 2.1");
    assert_eq!(posts_with_users[1].1.name, "User 2");

    // Post 1.2 -> User 1
    assert_eq!(posts_with_users[2].0.title, "Post 1.2");
    assert_eq!(posts_with_users[2].1.name, "User 1");
}

#[test]
fn test_many_to_many_eager_loading_mysql() {
    let mut conn = setup_db();

    // --- Post 1 with Tag 1, Tag 2 ---
    diesel::insert_into(users::table).values(users::name.eq("User 1")).execute(&mut conn).unwrap();
    let user_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(posts::table).values((posts::user_id.eq(user_id), posts::title.eq("Post 1"))).execute(&mut conn).unwrap();
    let post1_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    diesel::insert_into(tags::table).values(tags::name.eq("Tag 1")).execute(&mut conn).unwrap();
    let tag1_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(tags::table).values(tags::name.eq("Tag 2")).execute(&mut conn).unwrap();
    let tag2_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;

    diesel::insert_into(post_tags::table).values(&[
        (post_tags::post_id.eq(post1_id), post_tags::tag_id.eq(tag1_id)),
        (post_tags::post_id.eq(post1_id), post_tags::tag_id.eq(tag2_id)),
    ]).execute(&mut conn).unwrap();

    // --- Post 2 with Tag 2 ---
    diesel::insert_into(posts::table).values((posts::user_id.eq(user_id), posts::title.eq("Post 2"))).execute(&mut conn).unwrap();
    let post2_id = diesel::select(diesel::dsl::sql::<Unsigned<BigInt>>("LAST_INSERT_ID()")).first::<u64>(&mut conn).unwrap() as i32;
    diesel::insert_into(post_tags::table).values((post_tags::post_id.eq(post2_id), post_tags::tag_id.eq(tag2_id))).execute(&mut conn).unwrap();

    let all_posts = posts::table.order(posts::id.asc()).load::<Post>(&mut conn).unwrap();
    assert_eq!(all_posts.len(), 2);

    let posts_with_tags = Post::load_with_tags(all_posts, &mut conn).unwrap();

    assert_eq!(posts_with_tags.len(), 2);

    // Check Post 1
    assert_eq!(posts_with_tags[0].0.title, "Post 1");
    assert_eq!(posts_with_tags[0].1.len(), 2);
    // The order of tags is not guaranteed, so we check for presence instead of index
    assert!(posts_with_tags[0].1.iter().any(|t| t.name == "Tag 1"));
    assert!(posts_with_tags[0].1.iter().any(|t| t.name == "Tag 2"));


    // Check Post 2
    assert_eq!(posts_with_tags[1].0.title, "Post 2");
    assert_eq!(posts_with_tags[1].1.len(), 1);
    assert_eq!(posts_with_tags[1].1[0].name, "Tag 2");
}
