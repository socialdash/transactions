use std::net::SocketAddr;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper;
use hyper::Server;
use hyper::{service::Service, Body, Request, Response};
use r2d2;

use super::config::Config;
use super::utils::{log_and_capture_error, log_error, log_warn};
use utils::read_body;

mod controllers;
mod error;
mod requests;
mod responses;
pub mod utils;

use self::controllers::*;
use self::error::*;
use client::{BlockchainClient, BlockchainClientImpl, HttpClientImpl, KeysClient, KeysClientImpl};
use models::*;
use prelude::*;
use repos::{
    AccountsRepoImpl, BlockchainTransactionsRepoImpl, DbExecutorImpl, PendingBlockchainTransactionsRepoImpl, TransactionsRepoImpl,
    UsersRepoImpl,
};
use services::{AccountsServiceImpl, AuthServiceImpl, TransactionsServiceImpl, UsersServiceImpl};

#[derive(Clone)]
pub struct ApiService {
    server_address: SocketAddr,
    config: Config,
    db_pool: PgPool,
    cpu_pool: CpuPool,
    keys_client: Arc<dyn KeysClient>,
    blockchain_client: Arc<dyn BlockchainClient>,
}

impl ApiService {
    fn from_config(config: &Config) -> Result<Self, Error> {
        let server_address = format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(ectx!(try
                ErrorContext::Config,
                ErrorKind::Internal =>
                config.server.host,
                config.server.port
            ))?;
        let database_url = config.database.url.clone();
        let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
        let db_pool = r2d2::Pool::builder().build(manager).map_err(ectx!(try
            ErrorContext::Config,
            ErrorKind::Internal =>
            database_url
        ))?;
        let cpu_pool = CpuPool::new(config.cpu_pool.size);
        let client = HttpClientImpl::new(config);
        let keys_client = KeysClientImpl::new(&config, client.clone());
        let blockchain_client = BlockchainClientImpl::new(&config, client);

        Ok(ApiService {
            config: config.clone(),
            server_address,
            db_pool,
            cpu_pool,
            keys_client: Arc::new(keys_client),
            blockchain_client: Arc::new(blockchain_client),
        })
    }
}

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, http_body) = req.into_parts();
        let db_pool = self.db_pool.clone();
        let cpu_pool = self.cpu_pool.clone();
        let keys_client = self.keys_client.clone();
        let blockchain_client = self.blockchain_client.clone();
        let db_executor = DbExecutorImpl::new(db_pool.clone(), cpu_pool.clone());
        Box::new(
            read_body(http_body)
                .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let router = router! {
                        POST /v1/users => post_users,
                        GET /v1/users/me => get_users_me,
                        GET /v1/users/{user_id: UserId}/accounts => get_users_accounts,
                        POST /v1/accounts => post_accounts,
                        GET /v1/accounts/{account_id: AccountId} => get_accounts,
                        PUT /v1/accounts/{account_id: AccountId} => put_accounts,
                        DELETE /v1/accounts/{account_id: AccountId} => delete_accounts,
                        GET /v1/accounts/{account_id: AccountId}/balances => get_accounts_balances,
                        GET /v1/accounts/{account_id: AccountId}/transactions => get_accounts_transactions,
                        GET /v1/users/{user_id: UserId}/balances => get_users_balances,
                        GET /v1/users/{user_id: UserId}/transactions => get_users_transactions,
                        POST /v1/transactions => post_transactions,
                        GET /v1/transactions/{transaction_id: TransactionId} => get_transactions,
                        _ => not_found,
                    };

                    let auth_service = Arc::new(AuthServiceImpl::new(Arc::new(UsersRepoImpl), db_executor.clone()));
                    let users_service = Arc::new(UsersServiceImpl::new(Arc::new(UsersRepoImpl), db_executor.clone()));

                    let accounts_service = Arc::new(AccountsServiceImpl::new(
                        auth_service.clone(),
                        Arc::new(AccountsRepoImpl),
                        db_executor.clone(),
                        keys_client.clone(),
                    ));
                    let transactions_service = Arc::new(TransactionsServiceImpl::new(
                        auth_service.clone(),
                        Arc::new(TransactionsRepoImpl),
                        Arc::new(PendingBlockchainTransactionsRepoImpl),
                        Arc::new(BlockchainTransactionsRepoImpl),
                        Arc::new(AccountsRepoImpl),
                        db_executor.clone(),
                        keys_client,
                        blockchain_client,
                    ));

                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        users_service,
                        accounts_service,
                        transactions_service,
                    };

                    debug!("Received request {}", ctx);

                    router(ctx, parts.method.into(), parts.uri.path())
                }).or_else(|e| match e.kind() {
                    ErrorKind::BadRequest => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(400)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Bad request"}"#))
                            .unwrap())
                    }
                    ErrorKind::Unauthorized => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(401)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Unauthorized"}"#))
                            .unwrap())
                    }
                    ErrorKind::NotFound => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(404)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Not found"}"#))
                            .unwrap())
                    }
                    ErrorKind::UnprocessableEntity(errors) => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(422)
                            .header("Content-Type", "application/json")
                            .body(Body::from(format!("{}", errors)))
                            .unwrap())
                    }
                    ErrorKind::Internal => {
                        log_and_capture_error(e);
                        Ok(Response::builder()
                            .status(500)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Internal server error"}"#))
                            .unwrap())
                    }
                }),
        )
    }
}

pub fn start_server(config: Config) {
    hyper::rt::run(future::lazy(move || {
        ApiService::from_config(&config)
            .into_future()
            .and_then(move |api| {
                let api_clone = api.clone();
                let new_service = move || {
                    let res: Result<_, hyper::Error> = Ok(api_clone.clone());
                    res
                };
                let addr = api.server_address;
                let server = Server::bind(&api.server_address)
                    .serve(new_service)
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => addr));
                info!("Listening on http://{}", addr);
                server
            }).map_err(|e: Error| log_error(&e))
    }));
}
