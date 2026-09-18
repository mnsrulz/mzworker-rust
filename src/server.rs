use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{info, error};

use crate::options::options_query_service_server::{OptionsQueryService, OptionsQueryServiceServer};
use crate::options::{ExecuteQueryRequest, ExecuteQueryResponse};
use crate::query;

pub struct OptionsQueryServiceImpl {
    data_dir: PathBuf,
}

impl OptionsQueryServiceImpl {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn into_server(self) -> OptionsQueryServiceServer<Self> {
        OptionsQueryServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl OptionsQueryService for OptionsQueryServiceImpl {
    async fn execute_query(
        &self,
        request: Request<ExecuteQueryRequest>,
    ) -> Result<Response<ExecuteQueryResponse>, Status> {
        let req = request.into_inner();
        let data_dir = self.data_dir.clone();

        info!(query = %req.query, limit = req.limit, "Received query");

        match query::execute_query(req, data_dir).await {
            Ok(response) => {
                info!(rows = response.rows.len(), "Query completed");
                Ok(Response::new(response))
            }
            Err(e) => {
                error!(error = %e, "Query failed");
                Err(Status::invalid_argument(e.to_string()))
            }
        }
    }
}
