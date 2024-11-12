use serde::{Deserialize, Serialize};

// fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
// where
//     D: Deserializer<'de>,
//     T: Default + Deserialize<'de>,
// {
//     let value = Option::<T>::deserialize(deserializer)?;
//     Ok(value.unwrap_or_default())
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
  pub id: Option<String>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub created_at: Option<String>,
  pub modified_at: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListIssuesOutput {
  pub entities: Vec<Issue>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIssueOutput {
  pub entity: Issue
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertIssueInput {
  pub id: String,
  pub title: String,
  pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertIssueOutput {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteIssueOutput {
}
