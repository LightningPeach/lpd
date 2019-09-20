use std::io;
use std::boxed::Box;
use bitcoin_rpc_client::{Client, Error as BitcoinRpcError};
use grpc::Error as GrpcError;
use bitcoin::consensus::encode::Error as BitcoinEncodeError;
use std::fmt::Debug;

/// Represents locations in source files
#[derive(Clone)]
pub struct Location {
    pub column: u32,
    pub line: u32,
    pub file: &'static str,
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[macro_export]
macro_rules! get_location {
    () => {
        {
            $crate::error::Location {
                column: column!(),
                line: line!(),
                file: file!(),
            }
        }
    };
}

/// Represents an error
#[derive(Debug)]
pub struct Error {
    pub location: Location,
    pub description: String,
    pub error: ErrorWrapper,
}

#[macro_export]
macro_rules! new_io_error {
    ($err:expr, $description:expr, $target:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_io_error($err, Some($target)),
            }
        }
    };
    ($err:expr, $description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_io_error($err, None),
            }
        }
    }
}

#[macro_export]
macro_rules! new_bitcoin_rpc_error {
    ($err:expr, $description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_bitcoin_rpc_error($err),
            }
        }
    };
}

#[macro_export]
macro_rules! new_grpc_error {
    ($err:expr, $description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_grpc_error($err),
            }
        }
    };
}

#[macro_export]
macro_rules! new_bitcoin_encode_error {
    ($err:expr, $description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_bitcoin_encode_error($err),
            }
        }
    };
}

#[macro_export]
macro_rules! new_error {
    ($err:expr, $description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_error($err),
            }
        }
    };
}

#[macro_export]
macro_rules! new_other_error {
    ($description:expr) => {
        {
            use $crate::get_location;
            $crate::error::Error {
                location: get_location!(),
                description: $description.to_owned(),
                error: $crate::error::ErrorWrapper::new_other_error(),
            }
        }
    };
}

#[derive(Debug)]
pub enum ErrorWrapper {
    IO {
        inner: io::Error,
        target: Option<String>,
    },
    BitcoinRpc {
        inner: BitcoinRpcError,
    },
    Grpc {
        inner: GrpcError,
    },
    BitcoinEncode {
        inner: BitcoinEncodeError,
    },
    Error {
        inner: Box<Error>,
    },
    Other,
}

impl std::fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl ErrorWrapper {
    pub fn new_io_error(err: io::Error, target: Option<String>) -> ErrorWrapper {
        ErrorWrapper::IO {
            inner: err,
            target,
        }
    }

    pub fn new_bitcoin_rpc_error(err: BitcoinRpcError) -> ErrorWrapper {
        ErrorWrapper::BitcoinRpc {
            inner: err,
        }
    }

    pub fn new_grpc_error(err: GrpcError) -> ErrorWrapper {
        ErrorWrapper::Grpc {
            inner: err,
        }
    }

    pub fn new_bitcoin_encode_error(err: BitcoinEncodeError) -> ErrorWrapper {
        ErrorWrapper::BitcoinEncode {
            inner: err,
        }
    }

    pub fn new_error(err: Error) -> ErrorWrapper {
        ErrorWrapper::Error {
            inner: Box::new(err)
        }
    }

    pub fn new_other_error() -> ErrorWrapper {
        ErrorWrapper::Other
    }
}

impl std::error::Error for ErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorWrapper::IO{inner, target} => {
                Some(inner)
            },
            ErrorWrapper::BitcoinRpc {inner} => {
                Some(inner)
            },
            ErrorWrapper::Grpc {inner} => {
                Some(inner)
            },
            ErrorWrapper::BitcoinEncode {inner} => {
                Some(inner)
            },
            ErrorWrapper::Error {inner} => {
                Some(inner)
            },
            ErrorWrapper::Other => {
                None
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}