use axum::http::StatusCode;
use url::Url;

use crate::domain::models::{
    CreateCodelab, CreateFeedback, CreateMaterial, CreateQuiz, CreateStep, RegistrationPayload,
    UpdateStepsPayload,
};
use crate::utils::error::bad_request;

pub fn validate_codelab(payload: &CreateCodelab) -> Result<(), (StatusCode, String)> {
    validate_text(&payload.title, "title", 1, 200)?;
    validate_text(&payload.description, "description", 1, 4000)?;
    validate_text(&payload.author, "author", 1, 120)?;
    if let Some(guide) = &payload.guide_markdown {
        validate_text(guide, "guide_markdown", 0, 50_000)?;
    }
    Ok(())
}

pub fn validate_steps(payload: &UpdateStepsPayload) -> Result<(), (StatusCode, String)> {
    if payload.steps.is_empty() {
        return Err(bad_request("steps cannot be empty"));
    }
    if payload.steps.len() > 200 {
        return Err(bad_request("steps exceed limit"));
    }
    for step in &payload.steps {
        validate_step(step)?;
    }
    Ok(())
}

pub fn validate_step(step: &CreateStep) -> Result<(), (StatusCode, String)> {
    validate_text(&step.title, "step title", 1, 200)?;
    validate_text(&step.content_markdown, "step content", 1, 50_000)?;
    Ok(())
}

pub fn validate_registration(payload: &RegistrationPayload) -> Result<(), (StatusCode, String)> {
    validate_text(&payload.name, "name", 1, 80)?;
    validate_text(&payload.code, "code", 1, 64)?;
    Ok(())
}

pub fn validate_material(payload: &CreateMaterial) -> Result<(), (StatusCode, String)> {
    validate_text(&payload.title, "title", 1, 200)?;
    match payload.material_type.as_str() {
        "link" => {
            let link = payload
                .link_url
                .as_ref()
                .ok_or_else(|| bad_request("link_url is required for link material"))?;
            validate_text(link, "link_url", 1, 2048)?;
            let parsed = Url::parse(link).map_err(|_| bad_request("invalid link_url"))?;
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err(bad_request("link_url must be http or https"));
            }
        }
        "file" => {
            let path = payload
                .file_path
                .as_ref()
                .ok_or_else(|| bad_request("file_path is required for file material"))?;
            validate_text(path, "file_path", 1, 512)?;
            if !path.starts_with("/uploads/materials/") {
                return Err(bad_request("file_path must be an uploaded material"));
            }
        }
        _ => return Err(bad_request("invalid material_type")),
    }
    Ok(())
}

pub fn validate_quiz(quiz: &CreateQuiz) -> Result<(), (StatusCode, String)> {
    validate_text(&quiz.question, "question", 1, 500)?;
    if let Some(quiz_type) = &quiz.quiz_type {
        if quiz_type != "multiple_choice" && quiz_type != "descriptive" {
            return Err(bad_request("invalid quiz_type"));
        }
    }
    if quiz.quiz_type.as_deref() != Some("descriptive") {
        if quiz.options.len() < 2 || quiz.options.len() > 10 {
            return Err(bad_request("options must be between 2 and 10"));
        }
        if quiz.correct_answer < 0 || quiz.correct_answer as usize >= quiz.options.len() {
            return Err(bad_request("correct_answer out of range"));
        }
    }
    Ok(())
}

pub fn validate_feedback(payload: &CreateFeedback) -> Result<(), (StatusCode, String)> {
    validate_rating(&payload.difficulty, "difficulty")?;
    validate_rating(&payload.satisfaction, "satisfaction")?;
    if let Some(comment) = &payload.comment {
        validate_text(comment, "comment", 0, 1000)?;
    }
    Ok(())
}

pub fn validate_prompt(text: &str) -> Result<(), (StatusCode, String)> {
    validate_text(text, "prompt", 1, 100_000)
}

fn validate_rating(value: &str, field: &str) -> Result<(), (StatusCode, String)> {
    let parsed: i32 = value.parse().map_err(|_| bad_request("invalid rating"))?;
    if parsed < 1 || parsed > 5 {
        return Err(bad_request(&format!("{} out of range", field)));
    }
    Ok(())
}

fn validate_text(
    value: &str,
    field: &str,
    min: usize,
    max: usize,
) -> Result<(), (StatusCode, String)> {
    let trimmed = value.trim();
    if trimmed.len() < min {
        return Err(bad_request(&format!("{} is required", field)));
    }
    if trimmed.len() > max {
        return Err(bad_request(&format!("{} is too long", field)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_codelab_accepts_valid_payload() {
        let payload = CreateCodelab {
            title: "Title".to_string(),
            description: "Desc".to_string(),
            author: "Author".to_string(),
            is_public: Some(true),
            quiz_enabled: Some(false),
            require_quiz: Some(false),
            require_feedback: Some(false),
            guide_markdown: None,
        };
        assert!(validate_codelab(&payload).is_ok());
    }

    #[test]
    fn validate_registration_accepts_non_ascii_code() {
        let payload = RegistrationPayload {
            name: "Alice".to_string(),
            code: "바이브-코드 123".to_string(),
            email: None,
        };
        assert!(validate_registration(&payload).is_ok());
    }

    #[test]
    fn validate_material_requires_http_link() {
        let payload = CreateMaterial {
            title: "Spec".to_string(),
            material_type: "link".to_string(),
            link_url: Some("ftp://example.com".to_string()),
            file_path: None,
        };
        assert!(validate_material(&payload).is_err());
    }

    #[test]
    fn validate_quiz_rejects_invalid_options() {
        let quiz = CreateQuiz {
            question: "Q".to_string(),
            quiz_type: Some("multiple_choice".to_string()),
            options: vec!["A".to_string()],
            correct_answer: 0,
        };
        assert!(validate_quiz(&quiz).is_err());
    }
}
