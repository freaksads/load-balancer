use std::{sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::sleep,
};

struct Backend {
    address: &'static str,
    healthy: bool,
    connections: usize,
}

#[tokio::main]
async fn main() {
    let backends: Arc<RwLock<Vec<Backend>>> = Arc::new(RwLock::new(vec![
        Backend {
            address: "127.0.0.1:8001",
            healthy: false,
            connections: 0,
        },
        Backend {
            address: "127.0.0.1:8002",
            healthy: false,
            connections: 0,
        },
        Backend {
            address: "127.0.0.1:8003",
            healthy: false,
            connections: 0,
        },
    ]));

    let health_backends = Arc::clone(&backends);

    tokio::spawn(async move {
        loop {
            health_check(&health_backends).await;
            sleep(Duration::from_secs(3)).await;
        }
    });

    if let Ok(listener) = TcpListener::bind("127.0.0.1:8080").await {
        loop {
            let (mut client, _) = listener.accept().await.unwrap();
            let backends = Arc::clone(&backends);

            tokio::spawn(async move {
                let len = backends.read().await.len();
                let mut current: Option<usize> = None;

                for i in 0..len {
                    let health = {
                        let backends = backends.read().await;
                        backends[i].healthy
                    };

                    if health {
                        let address = {
                            let backends = backends.read().await;

                            if (current == None) {
                                current = Some(i);
                            } else {
                                if (backends[i].connections <= backends[current.unwrap()].connections) {
                                    current = Some(i);
                                }
                            }

                            backends[current.unwrap()].address
                        };

                        if let Ok(mut backend) = TcpStream::connect(address).await {
                            {
                                let backends = backends.write().await;
                                backends[current.unwrap()].connections += 1;
                            }

                            let mut buffer = [0; 4096];

                            let n = client.read(&mut buffer).await.unwrap();
                            backend.write_all(&buffer[..n]).await.unwrap();

                            let n = backend.read(&mut buffer).await.unwrap();
                            client.write_all(&buffer[..n]).await.unwrap();

                            {
                                let backends = backends.write().await;
                                backends[current.unwrap()].connections -= 1;
                            }

                            break;
                        }
                    }
                }
            });
        }
    }
}

async fn health_check(backends: &Arc<RwLock<Vec<Backend>>>) {
    let len = backends.read().await.len();

    for i in 0..len {
        let address = {
            let backends = backends.read().await;
            backends[i].address
        };

        let healthy = if let Ok(mut connection) = TcpStream::connect(address).await {
            let mut buffer = [0; 1024];

            if connection
                .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .is_ok()
            {
                if let Ok(n) = connection.read(&mut buffer).await {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    response.starts_with("HTTP/1.1 200")
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        let mut backends = backends.write().await;
        backends[i].healthy = healthy;
        // println!("Backend {address} healthy: {:?}", healthy)
    }
}
