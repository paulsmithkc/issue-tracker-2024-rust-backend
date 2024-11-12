use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("failed to parse json data: {0}")]
  SerdeJsonError(serde_json::Error),
  #[error("failed to parse database entity: {0}")]
  SerdeDynamoError(serde_dynamo::Error),
  #[error("aws_sdk_dynamodb error: {0}")]
  DynamoError(String),
  #[error("item not found")]
  NotFound,
}

impl From<serde_json::Error> for ApiError {
  fn from(err: serde_json::Error) -> Self {
    ApiError::SerdeJsonError(err)
  }
}

impl From<serde_dynamo::Error> for ApiError {
  fn from(err: serde_dynamo::Error) -> Self {
    ApiError::SerdeDynamoError(err)
  }
}

impl From<aws_sdk_dynamodb::Error> for ApiError {
  fn from(err: aws_sdk_dynamodb::Error) -> Self {
    ApiError::DynamoError(std::format!("{:?}", err))
  }
}

impl<E, R> From<aws_sdk_dynamodb::error::SdkError<E, R>> for ApiError
where
  E: std::fmt::Debug,
  R: std::fmt::Debug,
{
  fn from(err: aws_sdk_dynamodb::error::SdkError<E, R>) -> Self {
    ApiError::DynamoError(std::format!("{:?}", err))
  }
}
