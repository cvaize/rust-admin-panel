use actix_web::{error, web, Error, HttpResponse, Result};
use handlebars::Handlebars;
use serde_json::Value::Null;

pub async fn show(
    tmpl: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("pages/auth/forgot-password-confirm.hbs", &Null)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
