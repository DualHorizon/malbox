use axum::{
    http::{header::WWW_AUTHENTICATE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::head,
    Json,
};
use malbox_database::{DatabaseError, Error as SqlxError};
use std::collections::HashMap;
use std::{any, borrow::Cow};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Authentication required")]
    Unauthorized,

    #[error("User may not perform that action")]
    Forbidden,

    #[error("Request path not found")]
    NotFound,

    #[error("Error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    #[error("An internal server error occurred")]
    Internal(#[from] anyhow::Error),
}

impl Error {
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let errors = errors
            .into_iter()
            .map(|(k, v)| (k.into(), vec![v.into()]))
            .collect();

        Self::UnprocessableEntity { errors }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                let body = Json(serde_json::json!({ "errors": errors }));
                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }
            Self::Unauthorized => {
                let mut headers = HeaderMap::new();
                headers.insert(WWW_AUTHENTICATE, HeaderValue::from_static("Token"));
                (self.status_code(), headers, self.to_string()).into_response()
            }
            Self::Internal(ref err) => {
                tracing::error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occured",
                )
                    .into_response()
            }
            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}

impl From<SqlxError> for Error {
    fn from(err: SqlxError) -> Self {
        tracing::error!("Database error: {:?}", err);
        Error::Internal(anyhow::anyhow!("Database error occurred"))
    }
}

pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(&dyn DatabaseError) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(&dyn DatabaseError) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| {
            let error = e.into();
            if let Error::Internal(internal_error) = &error {
                if let Some(db_error) = internal_error.downcast_ref::<SqlxError>() {
                    if let SqlxError::Database(dbe) = db_error {
                        if dbe.constraint() == Some(name) {
                            return map_err(dbe.as_ref());
                        }
                    }
                }
            }
            error
        })
    }
}
