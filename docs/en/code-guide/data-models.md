# Data Models

This document explains the data models of Open Codelabs.

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

## Next Steps

- [Database Schema](../specification/database-schema.md)
- [Backend Examples](backend-examples.md)
