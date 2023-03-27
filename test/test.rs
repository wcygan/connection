extern crate connection;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        id: u32,
        name: String,
        payload: Vec<u8>,
    }

    use super::*;
    use connection::Connection;
    use tokio::net::TcpListener;

    async fn setup() -> (TcpListener, Connection) {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let conn = Connection::dial(addr).await.unwrap();
        (listener, conn)
    }

    #[tokio::test]
    async fn write_and_read_message() {
        let (server_listener, mut client_connection) = setup().await;
        let message = TestMessage {
            id: 123,
            name: "Test Message".to_string(),
            payload: vec![1, 2, 3, 4, 5],
        };

        client_connection.write(&message).await.unwrap();

        let mut server_connection = Connection::new(server_listener.accept().await.unwrap().0);
        let parsed_message: TestMessage = server_connection.read().await.unwrap().unwrap();
        assert_eq!(message, parsed_message);
    }

    #[tokio::test]
    async fn send_and_receive_hello_world() {
        let (server_listener, mut client_connection) = setup().await;
        client_connection.write(&"Hello, world!").await.unwrap();
        let mut server_connection = Connection::new(server_listener.accept().await.unwrap().0);
        let parsed_message: String = server_connection.read().await.unwrap().unwrap();
        assert_eq!("Hello, world!", parsed_message);
    }
}
