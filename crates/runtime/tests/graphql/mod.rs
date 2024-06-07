/*
Copyright 2024 The Spice.ai OSS Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use app::AppBuilder;

use async_graphql::{EmptyMutation, EmptySubscription, SimpleObject};
use async_graphql::{Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::post, Extension, Router};
use runtime::Runtime;
use spicepod::component::{dataset::Dataset, params::Params as DatasetParams};
use tokio::net::TcpListener;

use crate::{init_tracing, run_query_and_check_results, ValidateFn};

type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(SimpleObject)]
struct Post {
    id: String,
    title: String,
    content: String,
}

#[derive(SimpleObject)]
struct User {
    id: String,
    name: String,
    posts: Vec<Post>,
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn users(&self) -> Vec<User> {
        vec![
            User {
                id: "1".to_string(),
                name: "John Doe".to_string(),
                posts: vec![
                    Post {
                        id: "1".to_string(),
                        title: "Hello world".to_string(),
                        content: "Hello world".to_string(),
                    },
                    Post {
                        id: "2".to_string(),
                        title: "First post".to_string(),
                        content: "First post content".to_string(),
                    },
                ],
            },
            User {
                id: "2".to_string(),
                name: "Jane Doe".to_string(),
                posts: vec![
                    Post {
                        id: "3".to_string(),
                        title: "First post".to_string(),
                        content: "First post content".to_string(),
                    },
                    Post {
                        id: "4".to_string(),
                        title: "Second post".to_string(),
                        content: "Second post content".to_string(),
                    },
                ],
            },
            User {
                id: "3".to_string(),
                name: "Alice".to_string(),
                posts: vec![
                    Post {
                        id: "5".to_string(),
                        title: "First post".to_string(),
                        content: "First post content".to_string(),
                    },
                    Post {
                        id: "6".to_string(),
                        title: "Second post".to_string(),
                        content: "Second post content".to_string(),
                    },
                ],
            },
            User {
                id: "4".to_string(),
                name: "Bob".to_string(),
                posts: vec![
                    Post {
                        id: "7".to_string(),
                        title: "First post".to_string(),
                        content: "First post content".to_string(),
                    },
                    Post {
                        id: "8".to_string(),
                        title: "Second post".to_string(),
                        content: "Second post content".to_string(),
                    },
                ],
            },
        ]
    }
}

async fn graphql_handler(schema: Extension<ServiceSchema>, req: GraphQLRequest) -> GraphQLResponse {
    let response = schema.execute(req.into_inner()).await;

    response.into()
}

async fn start_server() -> Result<(tokio::sync::oneshot::Sender<()>, SocketAddr), String> {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema));

    let tcp_listener = TcpListener::bind("0.0.0.0:0").await.map_err(|e| {
        tracing::error!("Failed to bind to address: {e}");
        e.to_string()
    })?;
    let addr = tcp_listener.local_addr().map_err(|e| {
        tracing::error!("Failed to get local address: {e}");
        e.to_string()
    })?;

    tokio::spawn(async move {
        axum::serve(tcp_listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
            .unwrap_or_default();
    });

    Ok((tx, addr))
}

fn make_graphql_dataset(path: &str, name: &str) -> Dataset {
    let mut dataset = Dataset::new(format!("graphql:{path}"), name.to_string());
    let params = HashMap::from([
        ("json_path".to_string(), "data.users".to_string()),
        (
            "query".to_string(),
            "query { users { id name posts { id title content } } }".to_string(),
        ),
    ]);
    dataset.params = Some(DatasetParams::from_string_map(params));
    dataset
}

#[tokio::test]
async fn test_graphql() -> Result<(), String> {
    type QueryTests<'a> = Vec<(&'a str, Vec<&'a str>, Option<Box<ValidateFn>>)>;
    let _tracing = init_tracing(Some("integration=debug,info"));
    let (tx, addr) = start_server().await?;
    tracing::debug!("Server started at {}", addr);
    let app = AppBuilder::new("graphql_integration_test")
        .with_dataset(make_graphql_dataset(
            &format!("http://{addr}/graphql"),
            "test_graphql",
        ))
        .build();
    let mut rt = Runtime::new(Some(app), Arc::new(vec![])).await;

    tokio::select! {
        () = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            return Err("Timed out waiting for datasets to load".to_string());
        }
        () = rt.load_datasets() => {}
    }

    let queries: QueryTests = vec![
        (
            "SELECT * FROM test_graphql",
            vec![
                "+---------------+------------------------------------------------------+",
                "| plan_type     | plan                                                 |",
                "+---------------+------------------------------------------------------+",
                "| logical_plan  | TableScan: test_graphql projection=[id, name, posts] |",
                "| physical_plan | MemoryExec: partitions=1, partition_sizes=[4]        |",
                "|               |                                                      |",
                "+---------------+------------------------------------------------------+",
            ],
            Some(Box::new(|result_batches| {
                for batch in result_batches {
                    assert_eq!(batch.num_columns(), 3, "num_cols: {}", batch.num_columns());
                    assert_eq!(batch.num_rows(), 1, "num_rows: {}", batch.num_rows());
                }
            })),
        ),
        (
            "SELECT posts[1]['title'] from test_graphql",
            vec![
                "+---------------+--------------------------------------------------------------------------------------------------------------------------+",
                "| plan_type     | plan                                                                                                                     |",
                "+---------------+--------------------------------------------------------------------------------------------------------------------------+",
                "| logical_plan  | Projection: get_field(array_element(test_graphql.posts, Int64(1)), Utf8(\"title\")) AS test_graphql.posts[Int64(1)][title] |",
                "|               |   TableScan: test_graphql projection=[posts]                                                                             |",
                "| physical_plan | ProjectionExec: expr=[get_field(array_element(posts@0, 1), title) as test_graphql.posts[Int64(1)][title]]                |",
                "|               |   MemoryExec: partitions=1, partition_sizes=[4]                                                                          |",
                "|               |                                                                                                                          |",
                "+---------------+--------------------------------------------------------------------------------------------------------------------------+"
            ],
            Some(Box::new(|result_batches| {
                for batch in result_batches {
                    assert_eq!(batch.num_columns(), 1, "num_cols: {}", batch.num_columns());
                    assert_eq!(batch.num_rows(), 1, "num_rows: {}", batch.num_rows());
                }
            })),
        ),
    ];

    for (query, expected_plan, validate_result) in queries {
        run_query_and_check_results(&mut rt, query, &expected_plan, validate_result).await?;
    }

    tx.send(()).map_err(|()| {
        tracing::error!("Failed to send shutdown signal");
        "Failed to send shutdown signal".to_string()
    })?;

    Ok(())
}