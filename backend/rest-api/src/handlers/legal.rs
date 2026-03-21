use axum::extract::Query;

#[derive(Debug, serde::Deserialize)]
pub struct LangQuery {
    pub lang: Option<String>,
}

pub async fn legal_privacy(Query(q): Query<LangQuery>) -> axum::response::Html<&'static str> {
    if q.lang.as_deref() == Some("en") {
        axum::response::Html(include_str!("../../templates/privacy-en.html"))
    } else {
        axum::response::Html(include_str!("../../templates/privacy-fr.html"))
    }
}

pub async fn legal_terms(Query(q): Query<LangQuery>) -> axum::response::Html<&'static str> {
    if q.lang.as_deref() == Some("en") {
        axum::response::Html(include_str!("../../templates/terms-en.html"))
    } else {
        axum::response::Html(include_str!("../../templates/terms-fr.html"))
    }
}

pub async fn legal_mentions(Query(q): Query<LangQuery>) -> axum::response::Html<&'static str> {
    if q.lang.as_deref() == Some("en") {
        axum::response::Html(include_str!("../../templates/mentions-legales-en.html"))
    } else {
        axum::response::Html(include_str!("../../templates/mentions-legales-fr.html"))
    }
}
