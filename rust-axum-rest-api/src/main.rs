/*

Our goal is to build a high-performance REST API having all the CRUD operations (Create, Read, Update, Delete) for managing posts and users. Here's what we'll build:

GET /posts: Retrieve a list of all posts.
GET /posts/:id: Retrieve a specific post by its ID.
POST /posts: Create a new post.
PUT /posts: Update an existing post.
DELETE /posts: Delete an existing post.
POST /users: Create a new user.
We will be working with two database tables:

Posts: To store the post content and metadata.
Users: To manage the users who can create and interact with posts.

*/

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use sqlx::Pool;
use axum::{extract::Extension, routing::get, Json, Router};
use axum::routing::post;
use axum::extract::Path;
use tracing::{info, Level};
use tracing_subscriber;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Post {
    id: i32,
    user_id: Option<i32>,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize)]
struct UpdatePost {
    title: String,
    body: String,
    user_id: Option<i32>,
}

#[derive(Serialize)]
struct Message {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}
 
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}

/* Initial test for database connection

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _pool = PgPoolOptions::new().connect(&url).await?;
    println!("Connected to database");

    Ok(())
}
*/


// handler for "GET /" rest API endpoint
async fn root() -> &'static str {
    "Hello, world!"
}

// handler for "GET /posts" rest API endpoint
async fn get_posts(
    Extension(pool): Extension<Pool<Postgres>>
) -> Result<Json<Vec<Post>>, StatusCode> {
    let posts = sqlx::query_as!(Post, "SELECT id, title, body FROM posts")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(posts))
}

// handler for "GET /posts/:id" rest API endpoint
async fn get_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "SELECT id, user_id, title, body FROM posts WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
 
    Ok(Json(post))
}

// handler for Create a new post and return the created data
async fn create_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (user_id, title, body) VALUES ($1, $2, $3) RETURNING id, title, body, user_id",
        new_post.user_id,
        new_post.title,
        new_post.body
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
 
    Ok(Json(post))
}

// handler for Update a post and return the updated data
async fn update_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "UPDATE posts SET title = $1, body = $2, user_id = $3 WHERE id = $4 RETURNING id, user_id, title, body",
        updated_post.title,
        updated_post.body,
        updated_post.user_id,
        id
    )
    .fetch_one(&pool)
    .await;
 
    match post {
        Ok(post) => Ok(Json(post)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// This handler is a bit different as we delete a post we cannot return any data but we will return custom JSON response using the serde_json crate
async fn delete_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(&pool)
        .await;
 
    match result {
        Ok(_) => Ok(Json(serde_json::json! ({
            "message": "Post deleted successfully"
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_user): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
        new_user.username,
        new_user.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
 
    Ok(Json(user))
}


#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize tracing for logging with maximum level of tracing INFO
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // looading your environment variables from a .env file and connect to the database
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&url).await?;
    info!("Connected to the database!");
 
    // build anew router for our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/posts", get(get_posts).post(create_post))
        .route("/posts/:id", get(get_post).put(update_post).delete(delete_post))
        .route("/users", post(create_user))
        // extension layer
        .layer(Extension(pool));
 
    // run our app with hyper, listening globally on port 5000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    info!("Server is running on http://0.0.0.0:5000");
    axum::serve(listener, app).await.unwrap();
 
    Ok(())
}
