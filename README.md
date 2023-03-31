# Connection 💌

[<img alt="github" src="https://img.shields.io/badge/github-wcygan/connection-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcygan/connection)
[<img alt="crates.io" src="https://img.shields.io/crates/v/connection.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/connection)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-connection-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/connection)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcygan/connection/test.yml?branch=main&style=for-the-badge" height="20">](https://github.com/wcygan/connection/actions?query=branch%3Amain)
[![codecov](https://codecov.io/gh/wcygan/connection/branch/main/graph/badge.svg?token=CSWUVFE1R7)](https://codecov.io/gh/wcygan/connection)

A TCP-based connection that can send & receive serializable objects.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
connection = "0.2.5"
```

You can create a `Connection` by connecting like so:

```rust
use connection::Connection;

#[tokio::main]
async fn main() {
  let mut conn = Connection::connect("127.0.0.1:8080").await.unwrap();
}
```

You can use the `Connection` to send and receive serializable objects:

```rust
use connection::Connection;
use serde::{Serialize, Deserialize};

/// A (de)serializable type shared between client and server
#[derive(Serialize, Deserialize)]
struct Message {
  id: u32,
  data: String,
}

/// Code running client side
async fn client_side(mut client_conn: Connection) {
  let message = Message {
    id: 1,
    data: "Hello, world!".to_string(),
  };

  client_conn.write::<Message>(&message).await.unwrap();
}

/// Code running server side
async fn server_side(mut server_conn: Connection) {
  let message: Message = server_conn.read::<Message>().await.unwrap().unwrap();
}
```