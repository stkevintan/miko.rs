use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub email: Option<String>,
    pub current_password: String,
    pub new_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub path: Option<String>,
    pub name: Option<String>,
}
