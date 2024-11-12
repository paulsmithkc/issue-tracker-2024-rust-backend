mod errors;
mod models;
mod database;

use aws_sdk_dynamodb::Client;
use database::{delete_issue, get_issue, list_issues, upsert_issue};
use lambda_http::{http::StatusCode, run, service_fn, tracing, Error, IntoResponse, Request, RequestExt, RequestPayloadExt, Response};
use models::UpsertIssueInput;
use serde_json::json;

// Sources:
// https://www.cargo-lambda.info/guide/getting-started.html
// https://dynobase.dev/dynamodb-rust
// https://binaryheap.com/api-with-rust-and-lambda
// https://www.twilio.com/en-us/blog/build-high-performance-rest-apis-rust-axum

#[tokio::main]
async fn main() -> Result<(), Error> {
  tracing::init_default_subscriber();

  let client: Client = database::new_client().await;
  let client_ref: &Client = &client;

  run(service_fn(move |event: Request| async move {
    handler(client_ref, event).await
  })).await
}

fn trim_path(path: &str) -> &str {
  if path.starts_with("//") {
    return &path[1..];
  } else {
    return path;
  }
}

fn split_path(path: &str) -> (&str, &str) {
  if path.starts_with("/issue/") {
    let slug = &path[7..];
    return match slug {
      "list" => ("/issue/list", slug),
      "new" => ("/issue/new", slug),
      _ => ("/issue/{slug}", slug),
    }
  } else {
    return (path, "");
  }
}

async fn handler(client: &Client, event: Request) -> Result<impl IntoResponse, Error> {
  let method: &str = event.method().as_str();
  let path_raw: &str = event.uri().path();
  let (path, path_slug) = split_path(trim_path(path_raw));

  let slug =
    event.path_parameters_ref()
         .and_then(|params| params.first("slug"))
         .unwrap_or(path_slug);

  println!("\x1b[96m{method}\x1b[0m {path} {slug}");

  let f: &str = match path {
    "/healthCheck" => match method {
      "GET" => "healthCheck",
      _ => "405"
    },
    "/issue/list" => match method {
      "GET" => "issueList",
      _ => "405"
    },
    "/issue/new" => match method {
      "POST" => "issueInsert",
      _ => "405"
    },
    "/issue/{slug}" => match method {
      "GET" => "issueGet",
      "POST" => "issueUpdate",
      "DELETE" => "issueDelete",
      _ => "405"
    },
    _ => "404"
  };

  let status_code: StatusCode;
  let body: String;
  let content_type: &str = "application/json";

  match f {
    "healthCheck" => {
      status_code = StatusCode::OK;
      body = json!({ "success": true, "message": "OK" }).to_string();
    }
    "issueList" => {
      let output = list_issues(client).await;
      match output {
        Ok(r) => {
          status_code = StatusCode::OK;
          body = json!({ "success": true, "items": r.entities }).to_string();
        }
        Err(e) => {
          status_code = StatusCode::INTERNAL_SERVER_ERROR;
          body = json!({ "success": false, "error": format!("{}", e) }).to_string();
        }
      }
    }
    "issueGet" => {
      let output = get_issue(client, slug).await;
      match output {
        Ok(r) => {
          status_code = StatusCode::OK;
          body = json!({ "success": true, "item": r.entity }).to_string();
        }
        Err(e) => {
          status_code = StatusCode::INTERNAL_SERVER_ERROR;
          body = json!({ "success": false, "error": format!("{}", e) }).to_string();
        }
      }
    },
    "issueInsert" => {
      let data = event.payload::<UpsertIssueInput>();
      match data {
        Ok(d) => {
          let dd = &d.unwrap();
          let output = upsert_issue(client, dd).await;
          match output {
            Ok(_r) => {
              status_code = StatusCode::OK;
              body = json!({ "success": true, "message": format!("inserted {}", dd.id) }).to_string();
            }
            Err(e) => {
              status_code = StatusCode::INTERNAL_SERVER_ERROR;
              body = json!({ "success": false, "error": format!("{}", e) }).to_string();
            }
          }
        }
        Err(e) => {
          status_code = StatusCode::BAD_REQUEST;
          body = json!({ "success": false, "error": format!("{:?}", e) }).to_string();
        }
      }
    },
    "issueUpdate" => {
      let data = event.payload::<UpsertIssueInput>();
      match data {
        Ok(d) => {
          let dd = &d.unwrap();
          let output = upsert_issue(client, dd).await;
          match output {
            Ok(_r) => {
              status_code = StatusCode::OK;
              body = json!({ "success": true, "message": format!("updated {}", dd.id) }).to_string();
            }
            Err(e) => {
              status_code = StatusCode::INTERNAL_SERVER_ERROR;
              body = json!({ "success": false, "error": format!("{}", e) }).to_string();
            }
          }
        }
        Err(e) => {
          status_code = StatusCode::BAD_REQUEST;
          body = json!({ "success": false, "error": format!("{:?}", e) }).to_string();
        }
      }
    },
    "issueDelete" => {
      let output = delete_issue(client, slug).await;
      match output {
        Ok(_r) => {
          status_code = StatusCode::OK;
          body = json!({ "success": true, "message": format!("deleted {}", slug) }).to_string();
        }
        Err(e) => {
          status_code = StatusCode::INTERNAL_SERVER_ERROR;
          body = json!({ "success": false, "error": format!("{}", e) }).to_string();
        }
      }
    },
    "404" => {
      status_code = StatusCode::NOT_FOUND;
      body = json!({ "success": false, "message": format!("{} {} NOT FOUND", method, path) }).to_string();
    }
    "405" => {
      status_code = StatusCode::METHOD_NOT_ALLOWED;
      body = json!({ "success": false, "message": format!("{} {} METHOD NOT ALLOWED", method, path) }).to_string();
    }
    _ => {
      status_code = StatusCode::INTERNAL_SERVER_ERROR;
      body = json!({ "success": false, "message": format!("{} {} {} ROUTING ERROR", method, path, f) }).to_string();
    }
  }

  if status_code == 200 {
    println!("\x1b[96mSUCCESS\x1b[0m {status_code} {body}");
  } else {
    println!("\x1b[91mERROR\x1b[0m {status_code} {body}");
  }

  let resp= Response::builder()
    .status(status_code)
    .header("content-type", content_type)
    .body(body)
    .map_err(Box::new)?;

  Ok(resp)
}

