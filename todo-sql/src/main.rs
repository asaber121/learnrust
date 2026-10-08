use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};

#[derive(Serialize, FromRow)]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

fn internal(e: sqlx::Error) -> StatusCode {
    eprintln!("database error: {e}");
    StatusCode::INTERNAL_SERVER_ERROR
}

// GET /todos
async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos ORDER BY id")
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;
    Ok(Json(todos))
}

// POST /todos
async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title) VALUES (?) RETURNING id, title, done",
    )
    .bind(payload.title)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    Ok((StatusCode::CREATED, Json(todo)))
}

// GET /todos/{id}
async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    todo.map(Json).ok_or(StatusCode::NOT_FOUND)
}

// PUT /todos/{id}/done
async fn mark_done(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET done = 1 WHERE id = ? RETURNING id, title, done",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;
    todo.map(Json).ok_or(StatusCode::NOT_FOUND)
}

// DELETE /todos/{id}
async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://todos.db?mode=rwc")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done  BOOLEAN NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo).delete(delete_todo))
        .route("/todos/{id}/done", put(mark_done))
        .with_state(AppState { db: pool });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}