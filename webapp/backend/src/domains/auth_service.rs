use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::ImageReader;

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};

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
        let icon_width = 500;
        let icon_height = 500;

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

        // 画像サイズを含むファイル名でリサイズ済み画像を保存
        let output_name = format!(
            "resized_{}_{}_{}",
            icon_width, icon_height, profile_image_name
        );
        let output_path = Path::new(&format!("images/user_profile/{}", output_name)).to_path_buf();
        let redirect_path = format!("/protected/{}", output_name);

        // NOTE: レギュレーションで禁止
        // 既にリサイズ済みの画像が存在するか確認
        // if output_path.exists() {
        // return Ok(redirect_path);
        // }

        self.resize_and_save_image(&path, &output_path, icon_width, icon_height)
            .await?;

        // X-Accel-Redirect で画像を返すので、パスだけ渡す
        Ok(redirect_path)
    }

    // NOTE: エラーハンドリングをさぼってる
    async fn resize_and_save_image(
        &self,
        path: &Path,
        output_path: &Path,
        width: u32,
        height: u32,
    ) -> Result<(), AppError> {
        let src_image = ImageReader::open(path).unwrap().decode().unwrap();

        let mut dst_image = Image::new(width, height, src_image.pixel_type().unwrap());

        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dst_image, None).unwrap();

        let mut result_buf = BufWriter::new(Vec::new());
        PngEncoder::new(&mut result_buf)
            .write_image(dst_image.buffer(), width, height, src_image.color().into())
            .unwrap();

        std::fs::write(output_path, result_buf.get_ref()).unwrap();

        Ok(())
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<bool, AppError> {
        let session = self
            .repository
            .find_session_by_session_token(session_token)
            .await?;

        Ok(session.is_valid)
    }
}
