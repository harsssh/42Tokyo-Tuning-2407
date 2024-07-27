use log::error;
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::models::user::{Dispatcher, Session, User};
use crate::utils::{generate_session_token, hash_password, verify_password};

use super::dto::auth::LoginResponseDto;

pub trait AuthRepository {
    async fn create_user(&self, username: &str, password: &str, role: &str)
        -> Result<(), AppError>;
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, AppError>;
    async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn create_dispatcher(&self, user_id: i32, area_id: i32) -> Result<(), AppError>;
    async fn find_dispatcher_by_id(&self, id: i32) -> Result<Option<Dispatcher>, AppError>;
    async fn find_dispatcher_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<Dispatcher>, AppError>;
    async fn find_profile_image_name_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<String>, AppError>;
    async fn authenticate_user(&self, username: &str, password: &str) -> Result<User, AppError>;
    async fn create_session(&self, user_id: i32, session_token: &str) -> Result<(), AppError>;
    async fn delete_session(&self, session_token: &str) -> Result<(), AppError>;
    async fn find_session_by_session_token(&self, session_token: &str)
        -> Result<Session, AppError>;
}

#[derive(Debug)]
pub struct AuthService<T: AuthRepository + std::fmt::Debug> {
    repository: T,
}

impl<T: AuthRepository + std::fmt::Debug> AuthService<T> {
    pub fn new(repository: T) -> Self {
        AuthService { repository }
    }

    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
        area: Option<i32>,
    ) -> Result<LoginResponseDto, AppError> {
        if role == "dispatcher" && area.is_none() {
            return Err(AppError::BadRequest);
        }

        if (self.repository.find_user_by_username(username).await?).is_some() {
            return Err(AppError::Conflict);
        }

        let hashed_password = hash_password(password).unwrap();

        self.repository
            .create_user(username, &hashed_password, role)
            .await?;

        let session_token = generate_session_token();

        match self.repository.find_user_by_username(username).await? {
            Some(user) => {
                self.repository
                    .create_session(user.id, &session_token)
                    .await?;
                match user.role.as_str() {
                    "dispatcher" => {
                        self.repository
                            .create_dispatcher(user.id, area.unwrap())
                            .await?;
                        let dispatcher = self
                            .repository
                            .find_dispatcher_by_user_id(user.id)
                            .await?
                            .unwrap();
                        Ok(LoginResponseDto {
                            user_id: user.id,
                            username: user.username,
                            session_token,
                            role: user.role,
                            dispatcher_id: Some(dispatcher.id),
                            area_id: Some(dispatcher.area_id),
                        })
                    }
                    _ => Ok(LoginResponseDto {
                        user_id: user.id,
                        username: user.username,
                        session_token,
                        role: user.role,
                        dispatcher_id: None,
                        area_id: None,
                    }),
                }
            }
            None => Err(AppError::InternalServerError),
        }
    }

    pub async fn login_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResponseDto, AppError> {
        let user = self
            .repository
            .find_user_by_username(username)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if !verify_password(&user.password, password).unwrap() {
            return Err(AppError::Unauthorized);
        }

        let session_token = generate_session_token();
        self.repository
            .create_session(user.id, &session_token)
            .await?;

        let dispatcher_id: Option<i32>;
        let area_id: Option<i32>;

        if user.role == "dispatcher" {
            let dispatcher = self
                .repository
                .find_dispatcher_by_user_id(user.id)
                .await?
                .ok_or(AppError::InternalServerError)?;
            dispatcher_id = Some(dispatcher.id);
            area_id = Some(dispatcher.area_id);
        } else {
            dispatcher_id = None;
            area_id = None;
        }

        Ok(LoginResponseDto {
            user_id: user.id,
            username: user.username,
            session_token,
            role: user.role.clone(),
            dispatcher_id,
            area_id,
        })
    }

    pub async fn logout_user(&self, session_token: &str) -> Result<(), AppError> {
        self.repository.delete_session(session_token).await?;
        Ok(())
    }

    // NOTE: 戻り値は Path が適切かもしれない
    pub async fn get_resized_profile_image_path(&self, user_id: i32) -> Result<String, AppError> {
        let profile_image_name = match self
            .repository
            .find_profile_image_name_by_user_id(user_id)
            .await
        {
            Ok(Some(name)) => name,
            Ok(None) => return Err(AppError::NotFound),
            Err(_) => return Err(AppError::NotFound),
        };

        let path: PathBuf =
            Path::new(&format!("images/user_profile/{}", profile_image_name)).to_path_buf();

        let output_path = Path::new(&format!(
            "images/user_profile/resized_{}",
            profile_image_name
        ))
        .to_path_buf();
        let redirect_path = format!("/protected/resized_{}", profile_image_name);

        // 既にリサイズ済みの画像が存在するか確認
        if output_path.exists() {
            return Ok(redirect_path);
        }

        let img = image::open(path).map_err(|e| {
            error!("画像ファイルの読み込みに失敗しました: {:?}", e);
            AppError::InternalServerError
        })?;

        let resized = img.resize(500, 500, image::imageops::FilterType::Lanczos3);

        resized.save(&output_path).map_err(|e| {
            error!("画像ファイルの保存に失敗しました: {:?}", e);
            AppError::InternalServerError
        })?;

        // X-Accel-Redirect で画像を返すので、パスだけ渡す
        Ok(redirect_path)
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<bool, AppError> {
        let session = self
            .repository
            .find_session_by_session_token(session_token)
            .await?;

        Ok(session.is_valid)
    }
}
