use axum::{routing::get, routing::post,routing::put, Router};
use axum::extract::{Path, Query, Json, State};
use serde::{Deserialize, Serialize};
use axum::http::StatusCode;
use std::sync::{Arc, Mutex};

async fn index() -> String {
    String::from("Hello world")
}

#[derive(Clone, Serialize)]
struct Todo {
    id: u32,
    title: String,
    done: bool
}

#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}


async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    Json(todos.clone())
}

async fn create_todo(State(state): State<AppState>, Json(payload): Json<CreateTodo>) -> (StatusCode, Json<Todo>) {
    let mut todos = state.todos.lock().unwrap();
    let id = todos.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    let todo = Todo {
        id,
        title: payload.title,
        done: false
    };

    todos.push(todo.clone());

    (StatusCode::CREATED, Json(todo))
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Todo>, StatusCode> {
    let todos = state.todos.lock().unwrap();
    todos
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mark_done(
    State(state): State<AppState>,
    Path(id): Path<u32>
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = state.todos.lock().unwrap();
    let todo: &mut Todo = todos.iter_mut().find(|x| x.id == id).ok_or(StatusCode::NOT_FOUND)?;
    todo.done = true;
    Ok(Json(todo.clone()))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<u32>
) -> StatusCode {
    let mut todos = state.todos.lock().unwrap();
    let before = todos.len();

    todos.retain(|t| t.id != id);

    if todos.len() < before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo).delete(delete_todo))
        .route("/todos/{id}/done", put(mark_done))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server is running on port 3000");
    axum::serve(listener, app).await.unwrap();
}