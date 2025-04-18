use crate::app::controllers::web::auth::reset_password::CODE_LEN;
use crate::app::controllers::web::{DefaultForm, Field, FormData};
use crate::app::validator::rules::confirmed::Confirmed;
use crate::app::validator::rules::email::Email;
use crate::app::validator::rules::length::MinMaxLengthString;
use crate::app::validator::rules::required::Required;
use crate::{model_redis_impl, FlashService, Session};
use crate::{
    Alert, AppService, AuthService, TemplateService, Translator, TranslatorService, ALERTS_KEY,
};
use actix_web::web::{Data, Form, Query, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static DATA_KEY: &str = "page.reset_password_confirm.form.data";

model_redis_impl!(FormData<ResetPasswordConfirmFields>);

#[derive(Deserialize)]
pub struct ResetPasswordConfirmQuery {
    pub email: Option<String>,
    pub code: Option<String>,
}

pub async fn show(
    req: HttpRequest,
    session: Session,
    query: Query<ResetPasswordConfirmQuery>,
    flash_service: Data<FlashService>,
    tmpl_service: Data<TemplateService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let flash_service = flash_service.get_ref();
    let tmpl_service = tmpl_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();

    let query = query.into_inner();
    let (lang, locale, locales) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service);

    let alerts: Vec<Alert> = flash_service
        .all_throw_http(&session, ALERTS_KEY)?
        .unwrap_or(vec![]);
    let form_data: FormData<ResetPasswordConfirmFields> = flash_service
        .all_throw_http(&session, DATA_KEY)?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(DefaultForm::empty());

    let fields = form.fields.unwrap_or(ResetPasswordConfirmFields::empty());

    let mut email_field = fields.email.unwrap_or(Field::empty());
    let mut code_field = fields.code.unwrap_or(Field::empty());
    let password_field = fields.password.unwrap_or(Field::empty());
    let confirm_password_field = fields.confirm_password.unwrap_or(Field::empty());

    if query.email.is_none() || query.code.is_none() {
        return Ok(HttpResponse::SeeOther()
            .insert_header((
                http::header::LOCATION,
                http::HeaderValue::from_static("/reset-password"),
            ))
            .finish());
    }

    if let Some(email) = query.email {
        email_field.value = Some(email.to_owned());
    }

    if let Some(code) = query.code {
        code_field.value = Some(code.to_owned());
    }

    let ctx = json!({
        "title": translator.simple("auth.page.reset_password_confirm.title"),
        "locale": locale,
        "locales": locales,
        "alerts": alerts,
        "dark_mode": app_service.dark_mode(&req),
        "back": {
            "label": translator.simple("auth.page.reset_password_confirm.back.label"),
            "href": "/reset-password",
        },
        "form": {
            "action": "/reset-password-confirm",
            "method": "post",
            "header": translator.simple("auth.page.reset_password_confirm.form.header"),
            "fields": [
                {
                    "name": "code",
                    "type": "hidden",
                    "value": code_field.value,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.email.label"),
                    "type": "email",
                    "name": "email",
                    "readonly": "readonly",
                    "value": email_field.value,
                    "errors": email_field.errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.password.label"),
                    "type": "password",
                    "name": "password",
                    "value": password_field.value,
                    "errors": password_field.errors,
                },
                {
                    "label": translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label"),
                    "type": "password",
                    "name": "confirm_password",
                    "value": confirm_password_field.value,
                    "errors": confirm_password_field.errors,
                }
            ],
            "submit": {
                "label": translator.simple("auth.page.reset_password_confirm.form.submit.label"),
            },
            "errors": form.errors,
        },
    });

    let s = tmpl_service.render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn invoke(
    req: HttpRequest,
    session: Session,
    data: Form<ResetPasswordConfirmData>,
    flash_service: Data<FlashService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
    auth_service: Data<AuthService<'_>>,
) -> Result<impl Responder, Error> {
    let flash_service = flash_service.get_ref();
    let app_service = app_service.get_ref();
    let translator_service = translator_service.get_ref();
    let auth_service = auth_service.get_ref();

    let data: &ResetPasswordConfirmData = data.deref();

    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    let (lang, _, _) = app_service.locale(Some(&req), Some(&session), None);
    let translator = Translator::new(&lang, translator_service);
    let email_str = translator.simple("auth.page.reset_password_confirm.form.fields.email.label");
    let password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.password.label");
    let confirm_password_str =
        translator.simple("auth.page.reset_password_confirm.form.fields.confirm_password.label");

    let email_errors: Vec<String> = Required::validated(&translator, &data.email, |value| {
        Email::validate(&translator, value, &email_str)
    });
    let password_errors: Vec<String> = Required::validated(&translator, &data.password, |value| {
        MinMaxLengthString::validate(&translator, value, 4, 255, &password_str)
    });
    let mut confirm_password_errors: Vec<String> =
        Required::validated(&translator, &data.confirm_password, |value| {
            MinMaxLengthString::validate(&translator, value, 4, 255, &confirm_password_str)
        });
    let code_errors: Vec<String> = Required::validated(&translator, &data.code, |value| {
        MinMaxLengthString::validate(&translator, value, CODE_LEN, CODE_LEN, "code")
    });

    if password_errors.len() == 0 && confirm_password_errors.len() == 0 {
        let mut password_errors2: Vec<String> = Confirmed::validate(
            &translator,
            data.password.as_ref().unwrap(),
            data.confirm_password.as_ref().unwrap(),
            &password_str,
        );
        confirm_password_errors.append(&mut password_errors2);
    }

    let mut is_redirect_to_reset_password = false;
    let mut is_redirect_to_login = false;

    let is_valid = email_errors.len() == 0
        && code_errors.len() == 0
        && password_errors.len() == 0
        && confirm_password_errors.len() == 0;

    let d_ = "".to_string();
    let email = data.email.as_ref().unwrap_or(&d_);
    let code = data.code.as_ref().unwrap_or(&d_);
    let password = data.password.as_ref().unwrap_or(&d_);

    if is_valid {
        let is_code_equal: bool = auth_service
            .is_equal_reset_password_code(email, code)
            .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;

        if is_code_equal {
            let alert_str = translator.simple("auth.alert.confirm.success");
            alerts.push(Alert::success(alert_str));

            is_redirect_to_login = true;

            auth_service
                .update_password_by_email(email, password)
                .map_err(|_| error::ErrorInternalServerError("AuthService error"))?;
        } else {
            let alert_str = translator.simple("auth.alert.confirm.code_not_equal");
            alerts.push(Alert::error(alert_str));

            is_redirect_to_reset_password = true;
        }
    }

    let form_data = FormData {
        form: Some(DefaultForm {
            fields: Some(ResetPasswordConfirmFields {
                email: Some(Field {
                    value: data.email.to_owned(),
                    errors: Some(email_errors),
                }),
                code: Some(Field {
                    value: data.code.to_owned(),
                    errors: Some(code_errors),
                }),
                password: Some(Field {
                    value: data.password.to_owned(),
                    errors: Some(password_errors),
                }),
                confirm_password: Some(Field {
                    value: data.confirm_password.to_owned(),
                    errors: Some(confirm_password_errors),
                }),
            }),
            errors: Some(form_errors),
        }),
    };

    if is_valid {
        flash_service.delete_throw_http(&session, DATA_KEY)?;
    } else {
        flash_service.save_throw_http(&session, DATA_KEY, &form_data)?;
    }

    if alerts.len() == 0 {
        flash_service.delete_throw_http(&session, ALERTS_KEY)?;
    } else {
        flash_service.save_throw_http(&session, ALERTS_KEY, &alerts)?;
    }

    if is_redirect_to_reset_password {
        Ok(Redirect::to("/reset-password").see_other())
    } else if is_redirect_to_login {
        Ok(Redirect::to("/login").see_other())
    } else {
        let to = format!("/reset-password-confirm?code={}&email={}", code, email);
        Ok(Redirect::to(to).see_other())
    }
}

#[derive(Deserialize, Debug)]
pub struct ResetPasswordConfirmData {
    pub code: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordConfirmFields {
    pub code: Option<Field>,
    pub email: Option<Field>,
    pub password: Option<Field>,
    pub confirm_password: Option<Field>,
}

impl ResetPasswordConfirmFields {
    fn empty() -> Self {
        Self {
            code: None,
            email: None,
            password: None,
            confirm_password: None,
        }
    }
}
