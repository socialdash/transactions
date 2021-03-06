use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use serde_json;

use client::blockchain_gateway::ErrorKind as BlockchainClientErrorKind;
use client::exchange::ErrorKind as ExchangeClientErrorKind;
use client::fees::ErrorKind as FeesClientErrorKind;
use client::keys::ErrorKind as KeysClientErrorKind;
use repos::{Error as ReposError, ErrorKind as ReposErrorKind};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "service error - unauthorized")]
    Unauthorized,
    #[fail(display = "service error - malformed input")]
    MalformedInput,
    #[fail(display = "service error - invalid input, errors: {}", _0)]
    InvalidInput(String),
    #[fail(display = "service error - internal error")]
    Internal,
    #[fail(display = "service error - not found")]
    NotFound,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "service error source - r2d2")]
    R2D2,
    #[fail(display = "service error source - repos")]
    Repo,
    #[fail(display = "service error source - rabbit")]
    Lapin,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "service error context - no auth token received")]
    NoAuthToken,
    #[fail(display = "service error context - invalid auth token")]
    InvalidToken,
    #[fail(display = "service error context - no account found")]
    NoAccount,
    #[fail(display = "service error context - no transaction found")]
    NoTransaction,
    #[fail(display = "service error context - not enough funds")]
    NotEnoughFunds,
    #[fail(display = "service error context - invalid currency")]
    InvalidCurrency,
    #[fail(display = "service error context - exchange rate is required, but not found")]
    MissingExchangeRate,
    #[fail(display = "service error context - invalid utf8 bytes")]
    UTF8,
    #[fail(display = "service error context - failed to parse string to json")]
    Json,
    #[fail(display = "service error context - balance overflow")]
    BalanceOverflow,
    #[fail(display = "service error context - transaction between two dr accounts")]
    InvalidTransaction,
    #[fail(display = "service error context - invalid uuid")]
    InvalidUuid,
    #[fail(display = "service error context - operation not yet supproted")]
    NotSupported,
    #[fail(display = "service error context - invalid value")]
    InvalidValue,
    #[fail(display = "service error context - unexpected blockckhain transaction structure")]
    InvalidBlockchainTransactionStructure,
    #[fail(display = "service error context - unexpected transaction structure")]
    InvalidTransactionStructure,
    #[fail(display = "service error context - unexpected error in timer")]
    Timer,
    #[fail(display = "service error context - operations limit exceeded")]
    LimitExceeded,
    #[fail(display = "service error context - missing address in transaction")]
    MissingAddressInTx,
}

derive_error_impls!();

impl From<ReposError> for Error {
    fn from(e: ReposError) -> Error {
        let kind: ErrorKind = e.kind().into();
        e.context(kind).into()
    }
}

impl From<ReposErrorKind> for ErrorKind {
    fn from(e: ReposErrorKind) -> ErrorKind {
        match e {
            ReposErrorKind::AlreadyInTransaction | ReposErrorKind::Internal => ErrorKind::Internal,
            ReposErrorKind::Unauthorized => ErrorKind::Unauthorized,
            ReposErrorKind::Constraints(validation_errors) => {
                ErrorKind::InvalidInput(serde_json::to_string(&validation_errors).unwrap_or_default())
            }
        }
    }
}

impl From<KeysClientErrorKind> for ErrorKind {
    fn from(err: KeysClientErrorKind) -> Self {
        match err {
            KeysClientErrorKind::Internal => ErrorKind::Internal,
            KeysClientErrorKind::Unauthorized => ErrorKind::Internal,
            KeysClientErrorKind::MalformedInput => ErrorKind::Internal,
        }
    }
}

impl From<FeesClientErrorKind> for ErrorKind {
    fn from(err: FeesClientErrorKind) -> Self {
        match err {
            FeesClientErrorKind::Internal => ErrorKind::Internal,
            FeesClientErrorKind::Unauthorized => ErrorKind::Internal,
            FeesClientErrorKind::MalformedInput => ErrorKind::Internal,
        }
    }
}

impl From<BlockchainClientErrorKind> for ErrorKind {
    fn from(err: BlockchainClientErrorKind) -> Self {
        match err {
            BlockchainClientErrorKind::Internal => ErrorKind::Internal,
            BlockchainClientErrorKind::Unauthorized => ErrorKind::Internal,
            BlockchainClientErrorKind::MalformedInput => ErrorKind::Internal,
        }
    }
}

impl From<ExchangeClientErrorKind> for ErrorKind {
    fn from(err: ExchangeClientErrorKind) -> Self {
        match err {
            ExchangeClientErrorKind::Internal => ErrorKind::Internal,
            ExchangeClientErrorKind::Unauthorized => ErrorKind::Internal,
            ExchangeClientErrorKind::MalformedInput => ErrorKind::Internal,
            ExchangeClientErrorKind::Validation(s) => ErrorKind::InvalidInput(s),
        }
    }
}
