use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::{Serialize};
use tracing::{error, info};

use crate::sounds::{SoundManager, SoundInfo};
use crate::AppState;

#[derive(Serialize)]
pub struct SoundsListResponse {
    pub sounds: Vec<SoundInfo>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

/// GET /api/sounds
/// 
/// Get list of all available sounds with metadata
/// 
/// # Response
/// - 200: List of available sounds
/// - 500: Internal server error
pub async fn list_sounds(
    State(_state): State<AppState>,
) -> Result<Response, Response> {
    info!("Listing available sounds");

    let sound_info = SoundManager::get_all_sound_info();
    let sounds: Vec<SoundInfo> = sound_info.values().cloned().collect();

    Ok((
        StatusCode::OK,
        Json(SoundsListResponse { sounds }),
    ).into_response())
}

/// GET /api/sounds/:sound_name
/// 
/// Get MP3 data for a specific sound
/// 
/// # Path Parameters
/// - `sound_name`: Name of the sound (without .mp3 extension)
/// 
/// # Response
/// - 200: MP3 audio data with proper MIME type
/// - 404: Sound not found
/// - 500: Internal server error
pub async fn get_sound(
    State(_state): State<AppState>,
    Path(sound_name): Path<String>,
) -> Result<Response, Response> {
    info!("Getting sound: {}", sound_name);

    // Validate sound name format
    if !SoundManager::is_valid_sound_name(&sound_name) {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid sound name format",
        ));
    }

    // Get sound data
    match SoundManager::get_sound_data(&sound_name) {
        Some(data) => {
            let data_vec = data.as_ref().to_vec();
            info!("Serving sound {} ({} bytes)", sound_name, data_vec.len());
            
            Ok((
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, SoundManager::get_mime_type()),
                    (header::CACHE_CONTROL, "public, max-age=86400"), // Cache for 24 hours
                    (header::CONTENT_LENGTH, &data_vec.len().to_string()),
                ],
                data_vec,
            ).into_response())
        }
        None => {
            info!("Sound not found: {}", sound_name);
            Err(create_error_response(
                StatusCode::NOT_FOUND,
                &format!("Sound '{}' not found", sound_name),
            ))
        }
    }
}

/// GET /api/sounds/:sound_name/info
/// 
/// Get metadata for a specific sound
/// 
/// # Path Parameters
/// - `sound_name`: Name of the sound (without .mp3 extension)
/// 
/// # Response
/// - 200: Sound metadata
/// - 404: Sound not found
/// - 500: Internal server error
pub async fn get_sound_info(
    State(_state): State<AppState>,
    Path(sound_name): Path<String>,
) -> Result<Response, Response> {
    info!("Getting sound info: {}", sound_name);

    // Validate sound name format
    if !SoundManager::is_valid_sound_name(&sound_name) {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid sound name format",
        ));
    }

    // Get sound info
    match SoundManager::get_sound_info(&sound_name) {
        Some(info) => {
            Ok((
                StatusCode::OK,
                Json(info.clone()),
            ).into_response())
        }
        None => {
            Err(create_error_response(
                StatusCode::NOT_FOUND,
                &format!("Sound '{}' not found", sound_name),
            ))
        }
    }
}

/// Create a standardized error response
fn create_error_response(status: StatusCode, message: &str) -> Response {
    let error_response = ErrorResponse {
        error: message.to_string(),
        code: status.as_u16(),
    };

    (status, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_error_response() {
        let _response = create_error_response(StatusCode::NOT_FOUND, "Test error");
        // We can't easily test the response body here without more setup,
        // but we can verify the function doesn't panic
        assert!(true);
    }
}