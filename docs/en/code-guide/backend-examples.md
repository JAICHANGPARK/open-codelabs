# Backend Code Examples

This document introduces key code examples for the Rust Backend.

## Handler Example

### Create Codelab

```rust
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCodelab {
    pub title: String,
    pub description: String,
    pub author: String,
}

pub async fn create_codelab(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();

    let codelab = sqlx::query_as::<_, Codelab>(
        "INSERT INTO codelabs (id, title, description, author)
         VALUES (?, ?, ?, ?) RETURNING *"
    )
    .bind(&id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.author)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(codelab))
}
```

## WebSocket Example

```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_socket(socket, codelab_id, state)
    })
}
```

## Next Steps

- [API Reference](../specification/api-reference.md)
- [Data Models](data-models.md)
