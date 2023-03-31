#![allow(unused)]
//! A TCP-based connection that can send & receive serializable objects.
//!
//! # Examples
//!
//! ```
//! use connection::Connection;
//! use serde::{Serialize, Deserialize};
//!
//! /// A (de)serializable type shared between client and server
//! #[derive(Serialize, Deserialize)]
//! struct Message {
//!   id: u32,
//!   data: String,
//! }
//!
//! /// Code running client side
//! async fn client_side(mut client_conn: Connection) {
//!   let message = Message {
//!     id: 1,
//!     data: "Hello, world!".to_string(),
//!   };
//!
//!   client_conn.write::<Message>(&message).await.unwrap();
//! }
//!
//! /// Code running server side
//! async fn server_side(mut server_conn: Connection) {
//!   let message: Message = server_conn.read::<Message>().await.unwrap().unwrap();
//! }
use bytes::BytesMut;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Cursor, Error};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};

static DEFAULT_BUFFER_SIZE: usize = 4 * 1024;

/// The failure modes of a connection
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// An error encountered during IO
    #[error("`{0}`")]
    IoError(Error),
    /// An error encountered during (de)serialization
    #[error("`{0}`")]
    BincodeError(Box<bincode::Error>),
    /// An error encountered when the network connection is dropped
    #[error("`{0}`")]
    ConnectionReset(String),
}

/// A TCP connection that can be used to send and receive serializable values
pub struct Connection {
    buffer: BytesMut,
    stream: BufWriter<TcpStream>,
}

impl Connection {
    /// Connect to a socket address and return a new connection with the default buffer capacity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use connection::{Connection};
    /// use std::error::Error;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let mut conn = Connection::dial("127.0.0.1:8080").await?;
    ///
    ///     // Send a message
    ///     conn.write(&"Hello, world!").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn dial<A: ToSocketAddrs>(addr: A) -> Result<Connection, ConnectionError> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Connection::new(stream))
    }

    /// Connect to a socket address and return a new connection with a custom buffer capacity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use connection::{Connection};
    /// use std::error::Error;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let buffer_size = 4096;
    ///     let mut conn = Connection::dial_with_capacity("127.0.0.1:8080", buffer_size).await?;
    ///
    ///     // Send a message
    ///     conn.write(&"Hello, world!").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn dial_with_capacity<A: ToSocketAddrs>(
        addr: A,
        capacity: usize,
    ) -> Result<Connection, ConnectionError> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Connection::new_with_capacity(stream, capacity))
    }

    /// Create a new connection with the default buffer capacity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use connection::{Connection};
    /// use tokio::net::TcpStream;
    /// use std::error::Error;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     let mut conn = Connection::new(stream);
    ///
    ///     // Send a message
    ///     conn.write(&"Hello, world!").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(stream: TcpStream) -> Self {
        Self::new_with_capacity(stream, DEFAULT_BUFFER_SIZE)
    }

    /// Create a new connection with a custom buffer capacity
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use connection::{Connection};
    /// use tokio::net::TcpStream;
    /// use std::error::Error;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     let buffer_size = 4096;
    ///     let mut conn = Connection::new_with_capacity(stream, buffer_size);
    ///
    ///     // Send a message
    ///     conn.write(&"Hello, world!").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new_with_capacity(stream: TcpStream, capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            stream: BufWriter::new(stream),
        }
    }

    /// Write a serializable value into the stream
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde::{Serialize, Deserialize};
    /// use connection::{Connection};
    /// use std::error::Error;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Message { id: usize, text: String }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let mut conn = Connection::dial("127.0.0.1:8080").await?;
    ///
    ///     // Create a message
    ///     let message = Message { id: 1, text: "Hello, world!".to_string() };
    ///
    ///     // Send a message
    ///     conn.write::<Message>(&message).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn write<T: Serialize>(&mut self, value: &T) -> Result<(), ConnectionError> {
        let buf = bincode::serialize(value)?;
        self.write_to_stream(&buf).await?;
        Ok(())
    }

    /// Reads from the socket until a complete message is received, or an error occurs
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde::{Serialize, Deserialize};
    /// use connection::{Connection};
    /// use std::error::Error;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Message { id: usize, text: String }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     // Connect to a peer
    ///     let mut conn = Connection::dial("127.0.0.1:8080").await?;
    ///
    ///     // Read a message
    ///     let message: Message = conn.read::<Message>().await?.unwrap();
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>, ConnectionError> {
        loop {
            if let Some(value) = self.parse_value()? {
                self.buffer.clear();
                return Ok(Some(value));
            }

            self.read_to_buffer().await?;
        }
    }

    /// Attempts to deserialize a T from the internal buffer.
    fn parse_value<T: DeserializeOwned>(&mut self) -> Result<Option<T>, ConnectionError> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match bincode::deserialize_from(&mut buf) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Write a byte slice into the stream
    async fn write_to_stream(&mut self, buf: &[u8]) -> Result<(), ConnectionError> {
        self.stream.write_all(buf).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Reads more bytes from the socket into the internal buffer
    async fn read_to_buffer(&mut self) -> Result<(), ConnectionError> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            return if self.buffer.is_empty() {
                Ok(())
            } else {
                Err(ConnectionError::ConnectionReset(
                    "connection reset by peer".into(),
                ))
            };
        }
        Ok(())
    }
}

impl From<std::io::Error> for ConnectionError {
    fn from(e: std::io::Error) -> Self {
        ConnectionError::IoError(e)
    }
}

impl From<Box<bincode::ErrorKind>> for ConnectionError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        ConnectionError::BincodeError(Box::new(e))
    }
}
