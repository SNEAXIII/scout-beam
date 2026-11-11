//! Extractors that parse *and* validate, so a handler never sees a payload it would have to check.

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::shared::error::ApiError;

/// `Json<T>` + `T::validate()`.
#[derive(Debug, Clone, Copy)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| json_rejection(&rejection))?;
        value.validate()?;
        Ok(Self(value))
    }
}

fn json_rejection(rejection: &JsonRejection) -> ApiError {
    ApiError::UnprocessableBody(rejection.body_text())
}
