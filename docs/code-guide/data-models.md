# 데이터 모델

Open Codelabs의 데이터 모델을 설명합니다.

## Codelab

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Codelab {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}
```

## Step

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Step {
    pub id: String,
    pub codelab_id: String,
    pub step_number: i32,
    pub title: String,
    pub content_markdown: String,
}
```

## Attendee

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendee {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub code: String,
    pub current_step: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
}
```

## 다음 단계

- [데이터베이스 스키마](../specification/database-schema.md)
- [Backend 예제](backend-examples.md)
