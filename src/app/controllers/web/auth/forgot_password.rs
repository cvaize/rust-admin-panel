use crate::{Alert, AlertService, AppService, SessionService, TemplateService, TranslatorService};
use actix_session::Session;
use actix_web::web::{Data, Form, Redirect};
use actix_web::{error, Error, HttpRequest, HttpResponse, Responder, Result};
use garde::Validate;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::ops::Deref;

static FORM_DATA_KEY: &str = "page.forgot_password.form.data";

pub async fn show(
    req: HttpRequest,
    session: Session,
    tmpl: Data<TemplateService>,
    session_service: Data<SessionService>,
    alert_service: Data<AlertService>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<HttpResponse, Error> {
    let lang = app_service.get_locale(Some(&req), Some(&session), None);

    let alerts = alert_service
        .get_ref()
        .get_and_remove_from_session(&session)
        .unwrap_or(Vec::new());

    let form_data: FormData = session_service
        .get_and_remove(&session, FORM_DATA_KEY)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?
        .unwrap_or(FormData::empty());

    let form = form_data.form.unwrap_or(LoginForm::empty());

    let fields = form.fields.unwrap_or(Fields::empty());

    let email_field = fields.email.unwrap_or(Field::empty());

    let dark_mode = app_service.get_ref().get_dark_mode(&req);
    let title_str = translator_service.translate(&lang, "auth.page.forgot_password.title");
    let back_str = translator_service.translate(&lang, "auth.page.forgot_password.back.label");
    let header_str = translator_service.translate(&lang, "auth.page.forgot_password.form.header");
    let email_str =
        translator_service.translate(&lang, "auth.page.forgot_password.form.fields.email.label");
    let submit_str =
        translator_service.translate(&lang, "auth.page.forgot_password.form.submit.label");
    let submit_text_str =
        translator_service.translate(&lang, "auth.page.forgot_password.form.submit.text");

    let ctx = json!({
        "title": title_str,
        "lang": lang,
        "alerts": alerts,
        "dark_mode": dark_mode,
        "back": {
            "label": back_str,
            "href": "/login",
        },
        "form": {
            "action": "/forgot-password",
            "method": "post",
            "header": header_str,
            "fields": [
                {
                    "label": email_str,
                    "type": "email",
                    "name": "email",
                    "value": email_field.value,
                    "errors": email_field.errors,
                }
            ],
            "submit": {
                "label": submit_str,
                "text": submit_text_str
            },
            "errors": form.errors,
        },
    });

    let s = tmpl.get_ref().render_throw_http("pages/auth.hbs", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
pub async fn send_email(
    req: HttpRequest,
    session: Session,
    alert_service: Data<AlertService>,
    session_service: Data<SessionService>,
    data: Form<ForgotPasswordData>,
    app_service: Data<AppService>,
    translator_service: Data<TranslatorService>,
) -> Result<impl Responder, Error> {
    let data: &ForgotPasswordData = data.deref();

    let mut email_errors: Vec<String> = vec![];
    let mut alerts: Vec<Alert> = vec![];
    let form_errors: Vec<String> = vec![];

    if let Err(report) = data.validate() {
        for (path, error) in report.iter() {
            if path.to_string() == "email" {
                email_errors.push(error.message().to_string());
            }
        }
    } else {
        let lang = app_service.get_locale(Some(&req), Some(&session), None);
        let alert_str = translator_service.translate(&lang, "auth.alert.send_email.success");

        alerts.push(Alert::success(alert_str));
    };

    let email_field = Field {
        value: data.email.to_owned(),
        errors: Some(email_errors),
    };

    let fields = Fields {
        email: Some(email_field),
    };

    let form = LoginForm {
        fields: Some(fields),
        errors: Some(form_errors),
    };

    let form_data = FormData { form: Some(form) };

    session_service
        .get_ref()
        .insert(&session, FORM_DATA_KEY, &form_data)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    alert_service
        .get_ref()
        .insert_into_session(&session, &alerts)
        .map_err(|_| error::ErrorInternalServerError("Session error"))?;

    Ok(Redirect::to("/forgot-password").see_other())
}

#[derive(Validate, Deserialize, Debug)]
pub struct ForgotPasswordData {
    #[garde(required, inner(length(min = 1, max = 255)))]
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FormData {
    form: Option<LoginForm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginForm {
    fields: Option<Fields>,
    errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fields {
    email: Option<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    value: Option<String>,
    errors: Option<Vec<String>>,
}

impl FormData {
    fn empty() -> Self {
        Self { form: None }
    }
}

impl LoginForm {
    fn empty() -> Self {
        Self {
            fields: None,
            errors: None,
        }
    }
}

impl Fields {
    fn empty() -> Self {
        Self { email: None }
    }
}

impl Field {
    fn empty() -> Self {
        Self {
            value: None,
            errors: None,
        }
    }
}
