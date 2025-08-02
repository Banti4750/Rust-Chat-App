use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("HTTP Ping-Pong server listening on 127.0.0.1:8080");
    println!("Test with: curl -X POST http://127.0.0.1:8080 -d 'ping'");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 2048];

            match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("Client {} disconnected", addr);
                    return;
                },
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buf[0..n]);
                    println!("Received HTTP request from {}", addr);

                    // Parse HTTP request to extract body
                    let body = extract_http_body(&request);
                    let message = body.trim();

                    println!("Message: '{}'", message);

                    // Determine response based on input
                    let response_body = match message.to_lowercase().as_str() {
                        "ping" => "pong",
                        "pong" => "ping",
                        "hello" => "Hello there!",
                        "help" => "Commands: ping, pong, hello, help",
                        _ => &format!("Echo: {}", message)
                    };

                    // Create JSON response
                    let json_response = json!({
                        "input": message,
                        "output": response_body,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    // Send HTTP response
                    let http_response = format!(
                        "HTTP/1.1 200 OK\r\n\
                         Content-Type: application/json\r\n\
                         Access-Control-Allow-Origin: *\r\n\
                         Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                         Access-Control-Allow-Headers: Content-Type\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n\
                         \r\n\
                         {}",
                        json_response.to_string().len(),
                        json_response.to_string()
                    );

                    if let Err(e) = socket.write_all(http_response.as_bytes()).await {
                        eprintln!("Failed to write to socket; err = {:?}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

fn extract_http_body(request: &str) -> String {
    // Find the end of headers (double CRLF)
    if let Some(body_start) = request.find("\r\n\r\n") {
        request[body_start + 4..].to_string()
    } else if let Some(body_start) = request.find("\n\n") {
        request[body_start + 2..].to_string()
    } else {
        // If no body separator found, treat entire request as body
        // This handles simple cases where data is sent directly
        request.lines().last().unwrap_or("").to_string()
    }
}