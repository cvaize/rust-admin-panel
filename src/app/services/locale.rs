use crate::app::validator::rules::length::MinMaxLengthString;
use crate::{Config, KeyValueService, Locale, SessionService, User};
use crate::Session;
use actix_web::web::Data;
use actix_web::HttpRequest;
use http::header::ACCEPT_LANGUAGE;
use std::collections::HashMap;

static SESSION_LOCALE_KEY: &str = "locale";

#[derive(Debug)]
pub struct LocaleService {
    config: Data<Config>,
    session_service: Data<SessionService>,
    key_value_service: Data<KeyValueService>,
    locales: HashMap<String, Locale>,
    locales_codes: Vec<String>,
    locales_without_current: HashMap<String, Vec<Locale>>,
    locales_vec: Vec<Locale>,
}

impl LocaleService {
    pub fn new(config: Data<Config>, session_service: Data<SessionService>, key_value_service: Data<KeyValueService>) -> Self {
        let locales_vec = vec![
            Locale {
                code: "en".to_string(),
                short_name: "en".to_string(),
                full_name: "English".to_string(),
            },
            Locale {
                code: "ru".to_string(),
                short_name: "ru".to_string(),
                full_name: "Русский".to_string(),
            },
        ];
        let mut locales_codes: Vec<String> = Vec::new();
        let mut locales: HashMap<String, Locale> = HashMap::new();
        let mut locales_without_current: HashMap<String, Vec<Locale>> = HashMap::new();
        for locale in locales_vec.iter() {
            locales_codes.push(locale.code.to_string());
            locales.insert(locale.code.to_string(), locale.clone());
            let mut vec_without_current: Vec<Locale> = Vec::new();
            for locale2 in locales_vec.iter() {
                if locale.code != locale2.code {
                    vec_without_current.push(locale2.clone());
                }
            }
            locales_without_current.insert(locale.code.to_string(), vec_without_current);
        }
        Self {
            config,
            session_service,
            key_value_service,
            locales,
            locales_codes,
            locales_vec,
            locales_without_current,
        }
    }

    pub fn get_locale_ref(&self, code: &str) -> Option<&Locale> {
        self.locales.get(code)
    }

    pub fn get_locale_or_default_ref(&self, code: &str) -> &Locale {
        self.locales
            .get(code)
            .unwrap_or(self.locales.get(&self.config.get_ref().app.locale).unwrap())
    }

    pub fn get_locales_ref(&self) -> &Vec<Locale> {
        &self.locales_vec
    }

    pub fn get_locales_without_current_ref(&self, current_code: &str) -> Option<&Vec<Locale>> {
        self.locales_without_current.get(current_code)
    }

    pub fn get_locales_or_default_without_current_ref(&self, current_code: &str) -> &Vec<Locale> {
        self.locales_without_current.get(current_code).unwrap_or(
            self.locales_without_current
                .get(&self.config.get_ref().app.locale)
                .unwrap(),
        )
    }

    fn exists_locale_code_or_default(&self, key: String) -> String {
        if self.locales.contains_key(&key) {
            key
        } else {
            self.config.get_ref().app.locale.to_string()
        }
    }

    pub fn get_locales_codes_ref(&self) -> Vec<&str> {
        let mut codes = Vec::new();

        for locales_code in self.locales_codes.iter() {
            codes.push(locales_code.as_str());
        }

        codes
    }
    pub fn get_locale_code(
        &self,
        req: Option<&HttpRequest>,
        session: Option<&Session>,
        user: Option<&User>,
    ) -> String {
        if let Some(req) = req {
            if let Some(locale) = req.cookie(&self.config.get_ref().app.locale_cookie_key) {
                let locale = locale.value().to_string();
                if MinMaxLengthString::apply(&locale, 1, 6) {
                    return self.exists_locale_code_or_default(locale);
                }
            }
        }
        if let Some(session) = session {
            let key = self.session_service.get_ref().make_session_data_key(session, SESSION_LOCALE_KEY);
            let locale = self.key_value_service.get_ref().get(key);
            if let Ok(Some(locale)) = locale {
                if MinMaxLengthString::apply(&locale, 1, 6) {
                    return self.exists_locale_code_or_default(locale);
                }
            }
        }
        if let Some(user) = user {
            if let Some(locale) = &user.locale {
                if MinMaxLengthString::apply(&locale, 1, 6) {
                    return self.exists_locale_code_or_default(locale.to_string());
                }
            }
        }
        if let Some(req) = req {
            if let Some(header) = req.headers().get(ACCEPT_LANGUAGE) {
                let languages = accept_language::intersection(
                    header.to_str().unwrap_or(&self.config.get_ref().app.locale),
                    &self.get_locales_codes_ref(),
                );

                if let Some(locale) = languages.first() {
                    return self.exists_locale_code_or_default(locale.to_string());
                }
            }
        }

        self.config.get_ref().app.locale.to_string()
    }
}
