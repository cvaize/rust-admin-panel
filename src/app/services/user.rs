use crate::{
    AppError, HashService, MysqlRepository, PaginationResult, TranslatableError, TranslatorService,
    User, UserColumn, UserMysqlRepository, UserPaginateParams,
};
use actix_web::web::Data;
use actix_web::{error, Error};
use strum_macros::{Display, EnumString};

pub struct UserService {
    hash_service: Data<HashService>,
    user_repository: Data<UserMysqlRepository>,
}

impl UserService {
    pub fn new(
        hash_service: Data<HashService>,
        user_repository: Data<UserMysqlRepository>,
    ) -> Self {
        Self {
            hash_service,
            user_repository,
        }
    }

    pub fn first_by_id(&self, user_id: u64) -> Result<Option<User>, UserServiceError> {
        self.user_repository
            .get_ref()
            .first_by_id(user_id)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_id_throw_http(&self, user_id: u64) -> Result<User, Error> {
        let user = self
            .first_by_id(user_id)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    pub fn first_by_email(&self, email: &str) -> Result<Option<User>, UserServiceError> {
        self.user_repository
            .get_ref()
            .first_by_email(email)
            .map_err(|e| self.match_error(e))
    }

    pub fn first_by_email_throw_http(&self, email: &str) -> Result<User, Error> {
        let user = self
            .first_by_email(email)
            .map_err(|_| error::ErrorInternalServerError(""))?;
        if let Some(user) = user {
            return Ok(user);
        }
        Err(error::ErrorNotFound(""))
    }

    fn match_error(&self, e: AppError) -> UserServiceError {
        let error = e.to_string();

        if error.contains("Duplicate entry") {
            if error.contains(".email'") {
                return UserServiceError::DuplicateEmail;
            }
        }

        UserServiceError::Fail
    }

    pub fn create(&self, data: &User) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .insert_one(data)
            .map_err(|e| self.match_error(e))
    }

    pub fn update(
        &self,
        data: &User,
        columns: &Option<Vec<UserColumn>>,
    ) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .update_one(data, columns)
            .map_err(|e| self.match_error(e))
    }

    pub fn upsert(
        &self,
        data: &User,
        columns: &Option<Vec<UserColumn>>,
    ) -> Result<(), UserServiceError> {
        if data.id == 0 {
            self.user_repository
                .get_ref()
                .insert_one(data)
                .map_err(|e| self.match_error(e))
        } else {
            self.user_repository
                .get_ref()
                .update_one(data, columns)
                .map_err(|e| self.match_error(e))
        }
    }

    pub fn update_password_by_id(&self, id: u64, password: &str) -> Result<(), UserServiceError> {
        let hash_service = self.hash_service.get_ref();
        let password = hash_service.hash_password(password).map_err(|e| {
            log::error!("UserService::update_password_by_id - {e}");
            UserServiceError::PasswordHashFail
        })?;

        self.user_repository
            .get_ref()
            .update_password_by_id(id, &password)
            .map_err(|e| self.match_error(e))
    }

    pub fn update_password_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), UserServiceError> {
        let hash_service = self.hash_service.get_ref();
        let password = hash_service.hash_password(password).map_err(|e| {
            log::error!("UserService::update_password_by_id - {e}");
            UserServiceError::PasswordHashFail
        })?;

        self.user_repository
            .get_ref()
            .update_password_by_email(email, &password)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .delete_by_id(id)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_id_throw_http(&self, id: u64) -> Result<(), Error> {
        self.delete_by_id(id)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), UserServiceError> {
        self.user_repository
            .get_ref()
            .delete_by_ids(ids)
            .map_err(|e| self.match_error(e))
    }

    pub fn delete_by_ids_throw_http(&self, ids: &Vec<u64>) -> Result<(), Error> {
        self.delete_by_ids(ids)
            .map_err(|_| error::ErrorInternalServerError(""))
    }

    pub fn paginate(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, UserServiceError> {
        self.user_repository
            .get_ref()
            .paginate(params)
            .map_err(|e| self.match_error(e))
    }

    pub fn paginate_throw_http(
        &self,
        params: &UserPaginateParams,
    ) -> Result<PaginationResult<User>, Error> {
        self.paginate(params)
            .map_err(|_| error::ErrorInternalServerError(""))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum UserServiceError {
    DbConnectionFail,
    DuplicateEmail,
    PasswordHashFail,
    NotFound,
    Fail,
}

impl TranslatableError for UserServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::DbConnectionFail => {
                translator_service.translate(lang, "error.UserServiceError.DbConnectionFail")
            }
            Self::DuplicateEmail => {
                translator_service.translate(lang, "error.UserServiceError.DuplicateEmail")
            }
            Self::PasswordHashFail => {
                translator_service.translate(lang, "error.UserServiceError.PasswordHashFail")
            }
            Self::NotFound => translator_service.translate(lang, "error.UserServiceError.NotFound"),
            _ => translator_service.translate(lang, "error.UserServiceError.Fail"),
        }
    }
}
