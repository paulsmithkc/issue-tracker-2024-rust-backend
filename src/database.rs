use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;
use tracing::error;
use crate::errors::ApiError;
use crate::models::{DeleteIssueOutput, GetIssueOutput, Issue, ListIssuesOutput, UpsertIssueInput, UpsertIssueOutput};

pub async fn new_client() -> Client {
  let region_provider =
    aws_config::meta::region::RegionProviderChain::default_provider().or_else("us-east-1");
  let sdk_config =
    aws_config::from_env().region(region_provider).load().await;
  let config: aws_sdk_dynamodb::Config;

  let endpoint: String = std::env::var("DYNAMO_ENDPOINT").unwrap_or_default();
  if !endpoint.is_empty() {
    config =
      aws_sdk_dynamodb::config::Builder::from(&sdk_config)
        .endpoint_url(endpoint)
        .build();
  } else {
    config =
      aws_sdk_dynamodb::config::Builder::from(&sdk_config)
      .build();
  }

  return Client::from_conf(config)
}

pub async fn list_issues(client: &Client) -> Result<ListIssuesOutput, ApiError> {
  let output =
    client.scan()
          .table_name("issues")
          .limit(100)
          .send()
          .await?;

  match output.items {
    Some(items) => {
      let mut entities: Vec<Issue> = Vec::new();

      for i in items {
        let entity: Result<Issue, serde_dynamo::Error> = serde_dynamo::from_item(i);
        match entity {
          Ok(entity) => {
            entities.push(entity);
          }
          Err(e) => {
            error!("(Error)={:?}", e);
          }
        }
      }

      Ok(ListIssuesOutput { entities })
    }
    None => {
      Ok(ListIssuesOutput { entities: Vec::new() })
    }
  }
}

pub async fn get_issue(client: &Client, slug: &str) -> Result<GetIssueOutput, ApiError> {
  let output =
    client.get_item()
          .table_name("issues")
          .key("id", AttributeValue::S(slug.to_string()))
          .send()
          .await?;

  match output.item {
    Some(item) => {
      let entity: Issue = serde_dynamo::from_item(item)?;
      Ok(GetIssueOutput { entity })
    }
    None => {
      Err(ApiError::NotFound)
    }
  }
}

pub async fn upsert_issue(client: &Client, data: &UpsertIssueInput) -> Result<UpsertIssueOutput, ApiError> {
  let now = format!("{:?}", Utc::now());

  let output =
    client.update_item()
          .table_name("issues")
          .key("id", AttributeValue::S(data.id.to_string()))
          .update_expression("SET title = :title, description = :description, created_at = if_not_exists(created_at, :now), modified_at = :now")
          .expression_attribute_values(":title", AttributeValue::S(data.title.to_string()))
          .expression_attribute_values(":description", AttributeValue::S(data.description.to_string()))
          .expression_attribute_values(":now", AttributeValue::S(now))
          .send()
          .await;

  match output {
    Ok(_) => Ok(UpsertIssueOutput {}),
    Err(e) => Err(e.into())
  }
}

pub async fn delete_issue(client: &Client, slug: &str) -> Result<DeleteIssueOutput, ApiError> {
  let output =
    client.delete_item()
          .table_name("issues")
          .key("id", AttributeValue::S(slug.to_string()))
          .send()
          .await;

  match output {
    Ok(_) => Ok(DeleteIssueOutput {}),
    Err(e) => Err(e.into())
  }
}
