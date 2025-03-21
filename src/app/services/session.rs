use actix_session::Session;
use actix_session::SessionExt;
use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use serde_derive::{Deserialize, Serialize};

static SESSION_FLASH_DATA_KEY: &str = "app.session.flash_data.";
static SESSION_FLASH_DATA_COMMON_KEY: &str = "app.session.flash_data.common";
static SESSION_DARK_MODE_KEY: &str = "app.session.dark_mode";

pub struct SessionFlashService {
    pub session: Session,
}
pub struct SessionService {
    pub session: Session,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionFlashData {
    pub alerts: Option<Vec<SessionFlashAlert>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum SessionFlashAlert {
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct SessionFlashServiceError;

#[derive(Debug, Clone, Copy)]
pub struct SessionServiceError;

pub trait SessionFlashDataTrait {
    fn empty() -> Self;
}

impl SessionFlashDataTrait for SessionFlashData {
    fn empty() -> Self {
        Self { alerts: None }
    }
}

impl SessionFlashService {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    fn make_key(key: Option<&str>) -> String {
        match key {
            Some(k) => format!("{}{}", SESSION_FLASH_DATA_KEY, k),
            None => SESSION_FLASH_DATA_COMMON_KEY.to_string(),
        }
    }

    pub fn save<T>(&self, data: &T, key: Option<&str>) -> Result<(), SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::Serialize,
    {
        let json: String = serde_json::to_string(&data).map_err(|_| SessionFlashServiceError)?;
        self.session
            .insert(Self::make_key(key).as_str(), json)
            .map_err(|_| SessionFlashServiceError)?;
        Ok(())
    }

    pub fn read<T>(&self, key: Option<&str>) -> Result<T, SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::de::DeserializeOwned,
    {
        let result: Option<String> = self
            .session
            .get::<String>(Self::make_key(key).as_str())
            .map_err(|_| SessionFlashServiceError)?;

        match result {
            Some(str) => {
                let flash_data: T =
                    serde_json::from_str(&str).map_err(|_| SessionFlashServiceError)?;
                Ok(flash_data)
            }
            _ => Ok(T::empty()),
        }
    }

    pub fn read_and_forget<T>(&self, key: Option<&str>) -> Result<T, SessionFlashServiceError>
    where
        T: SessionFlashDataTrait + serde::de::DeserializeOwned,
    {
        let flash_data: T = self.read(key)?;
        self.session.remove(Self::make_key(key).as_str());
        Ok(flash_data)
    }
}

impl SessionService {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    pub fn dark_mode(&self) -> Result<bool, SessionServiceError> {
        Ok(1 == self
            .session
            .get::<u8>(SESSION_DARK_MODE_KEY)
            .map_err(|_| SessionServiceError)?
            .unwrap_or(0))
    }

    pub fn set_dark_mode(&self, dark_mode: bool) -> Result<(), SessionServiceError> {
        let dark_mode: u8 = if dark_mode { 1 } else { 0 };
        self.session
            .insert(SESSION_DARK_MODE_KEY, dark_mode)
            .map_err(|_| SessionServiceError)?;
        Ok(())
    }

    pub fn clear_dark_mode(&self) -> Result<(), SessionServiceError> {
        self.session
            .remove(SESSION_DARK_MODE_KEY);
        Ok(())
    }
}

impl FromRequest for SessionFlashService {
    type Error = Error;
    type Future = Ready<Result<SessionFlashService, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session: Session = req.get_session();

        let flash_service = SessionFlashService::new(session);
        ready(Ok(flash_service))
    }
}

impl FromRequest for SessionService {
    type Error = Error;
    type Future = Ready<Result<SessionService, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session: Session = req.get_session();

        let service = SessionService::new(session);
        ready(Ok(service))
    }
}
