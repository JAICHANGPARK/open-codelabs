# 백엔드 코드 예제

Rust Backend의 주요 코드 예제를 소개합니다.

## Handler 예제

### Codelab 생성

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

## WebSocket 예제

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

## 다음 단계

- [API 레퍼런스](../specification/api-reference.md)
- [데이터 모델](data-models.md)
