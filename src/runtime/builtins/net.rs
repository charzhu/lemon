//! Networking built-in functions

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::rc::Rc;
use std::time::Duration;

use crate::runtime::value::{RuntimeError, Value};

/// net::tcp_listen(addr: String) -> Result<TcpListener, Error>
pub fn builtin_net_tcp_listen(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let addr = args[0].as_string()?;
    match TcpListener::bind(addr) {
        Ok(listener) => Ok(Value::Ok(Box::new(Value::TcpListener(Rc::new(RefCell::new(
            listener,
        )))))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// net::tcp_accept(listener: TcpListener) -> Result<TcpStream, Error>
pub fn builtin_net_tcp_accept(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::TcpListener(listener) => {
            let listener = listener.borrow();
            match listener.accept() {
                Ok((stream, _addr)) => Ok(Value::Ok(Box::new(Value::TcpStream(Rc::new(
                    RefCell::new(stream),
                ))))),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }
        _ => Err(RuntimeError::type_error("TcpListener", args[0].type_name())),
    }
}

/// net::tcp_connect(addr: String) -> Result<TcpStream, Error>
pub fn builtin_net_tcp_connect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let addr = args[0].as_string()?;
    match TcpStream::connect(addr) {
        Ok(stream) => Ok(Value::Ok(Box::new(Value::TcpStream(Rc::new(RefCell::new(
            stream,
        )))))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// net::tcp_read(stream: TcpStream) -> Result<String, Error>
pub fn builtin_net_tcp_read(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::TcpStream(stream) => {
            let mut stream = stream.borrow_mut();
            let mut buffer = [0u8; 4096];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                    Ok(Value::Ok(Box::new(Value::String(data))))
                }
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }
        _ => Err(RuntimeError::type_error("TcpStream", args[0].type_name())),
    }
}

/// net::tcp_read_line(stream: TcpStream) -> Result<String, Error>
pub fn builtin_net_tcp_read_line(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::TcpStream(stream) => {
            let mut stream = stream.borrow_mut();
            let mut reader = BufReader::new(&mut *stream);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(_) => Ok(Value::Ok(Box::new(Value::String(line)))),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }
        _ => Err(RuntimeError::type_error("TcpStream", args[0].type_name())),
    }
}

/// net::tcp_write(stream: TcpStream, data: String) -> Result<(), Error>
pub fn builtin_net_tcp_write(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let data = args[1].as_string()?;

    match &args[0] {
        Value::TcpStream(stream) => {
            let mut stream = stream.borrow_mut();
            match stream.write_all(data.as_bytes()) {
                Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }
        _ => Err(RuntimeError::type_error("TcpStream", args[0].type_name())),
    }
}

/// net::tcp_close(stream: TcpStream) -> ()
pub fn builtin_net_tcp_close(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    // The handle will be dropped when the Rc goes out of scope
    // For explicit close, we just return unit
    match &args[0] {
        Value::TcpStream(_) | Value::TcpListener(_) => Ok(Value::Unit),
        _ => Err(RuntimeError::type_error("TcpStream or TcpListener", args[0].type_name())),
    }
}

/// net::tcp_set_timeout(stream: TcpStream, ms: int) -> Result<(), Error>
pub fn builtin_net_tcp_set_timeout(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let ms = args[1].as_int()?;
    let timeout = if ms > 0 {
        Some(Duration::from_millis(ms as u64))
    } else {
        None
    };

    match &args[0] {
        Value::TcpStream(stream) => {
            let stream = stream.borrow();
            if let Err(e) = stream.set_read_timeout(timeout) {
                return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
            }
            if let Err(e) = stream.set_write_timeout(timeout) {
                return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
            }
            Ok(Value::Ok(Box::new(Value::Unit)))
        }
        _ => Err(RuntimeError::type_error("TcpStream", args[0].type_name())),
    }
}

// ============================================================================
// Simple HTTP Implementation
// ============================================================================

/// http::get(url: String) -> Result<String, Error>
/// Simple HTTP GET request (HTTP/1.1, no HTTPS)
pub fn builtin_http_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let url = args[0].as_string()?;

    // Parse URL (simple parsing for http:// URLs)
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = if let Some(idx) = url.find('/') {
        (&url[..idx], &url[idx..])
    } else {
        (url, "/")
    };

    let (host, port) = if let Some(idx) = host_port.find(':') {
        (&host_port[..idx], host_port[idx + 1..].parse().unwrap_or(80))
    } else {
        (host_port, 80u16)
    };

    // Connect
    let addr = format!("{}:{}", host, port);
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => return Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    };

    // Set timeout
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

    // Send HTTP request
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: Lemon/0.1\r\n\r\n",
        path, host
    );

    if let Err(e) = stream.write_all(request.as_bytes()) {
        return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
    }

    // Read response
    let mut response = String::new();
    if let Err(e) = stream.read_to_string(&mut response) {
        return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
    }

    // Extract body (skip headers)
    if let Some(idx) = response.find("\r\n\r\n") {
        let body = &response[idx + 4..];
        Ok(Value::Ok(Box::new(Value::String(body.to_string()))))
    } else {
        Ok(Value::Ok(Box::new(Value::String(response))))
    }
}

/// http::post(url: String, body: String) -> Result<String, Error>
/// Simple HTTP POST request (HTTP/1.1, no HTTPS)
pub fn builtin_http_post(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let url = args[0].as_string()?;
    let body = args[1].as_string()?;

    // Parse URL
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = if let Some(idx) = url.find('/') {
        (&url[..idx], &url[idx..])
    } else {
        (url, "/")
    };

    let (host, port) = if let Some(idx) = host_port.find(':') {
        (&host_port[..idx], host_port[idx + 1..].parse().unwrap_or(80))
    } else {
        (host_port, 80u16)
    };

    // Connect
    let addr = format!("{}:{}", host, port);
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => return Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    };

    // Set timeout
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

    // Send HTTP request
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nUser-Agent: Lemon/0.1\r\n\r\n{}",
        path, host, body.len(), body
    );

    if let Err(e) = stream.write_all(request.as_bytes()) {
        return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
    }

    // Read response
    let mut response = String::new();
    if let Err(e) = stream.read_to_string(&mut response) {
        return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
    }

    // Extract body
    if let Some(idx) = response.find("\r\n\r\n") {
        let body = &response[idx + 4..];
        Ok(Value::Ok(Box::new(Value::String(body.to_string()))))
    } else {
        Ok(Value::Ok(Box::new(Value::String(response))))
    }
}

/// Helper to parse HTTP request from a stream
fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String, String), std::io::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse "GET /path HTTP/1.1"
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    let method = parts.get(0).unwrap_or(&"GET").to_string();
    let path = parts.get(1).unwrap_or(&"/").to_string();

    // Read headers until empty line
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        headers.push_str(&line);
    }

    // For POST, read body based on Content-Length
    let mut body = String::new();
    if method == "POST" {
        for header in headers.lines() {
            if header.to_lowercase().starts_with("content-length:") {
                if let Ok(len) = header[15..].trim().parse::<usize>() {
                    let mut body_bytes = vec![0u8; len];
                    reader.read_exact(&mut body_bytes)?;
                    body = String::from_utf8_lossy(&body_bytes).to_string();
                }
            }
        }
    }

    Ok((method, path, body))
}

/// Helper to write HTTP response
fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    body: &str,
) -> Result<(), std::io::Error> {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
        status, status_text, body.len(), body
    );

    stream.write_all(response.as_bytes())
}

/// http::serve_once(port: int) -> Result<(String, String, String), Error>
/// Accepts one HTTP connection and returns (method, path, body)
pub fn builtin_http_serve_once(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let port = args[0].as_int()?;
    let addr = format!("0.0.0.0:{}", port);

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => return Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    };

    // Accept one connection
    let (mut stream, _) = match listener.accept() {
        Ok(s) => s,
        Err(e) => return Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    };

    // Parse request
    let (method, path, body) = match parse_http_request(&mut stream) {
        Ok(r) => r,
        Err(e) => return Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    };

    // Return request info as a tuple
    Ok(Value::Ok(Box::new(Value::Tuple(vec![
        Value::String(method),
        Value::String(path),
        Value::String(body),
    ]))))
}

/// http::respond(stream: TcpStream, status: int, body: String) -> Result<(), Error>
pub fn builtin_http_respond(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::arity_mismatch(3, args.len()));
    }

    let status = args[1].as_int()? as u16;
    let body = args[2].as_string()?;

    match &args[0] {
        Value::TcpStream(stream) => {
            let mut stream = stream.borrow_mut();
            match write_http_response(&mut *stream, status, body) {
                Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
                Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
            }
        }
        _ => Err(RuntimeError::type_error("TcpStream", args[0].type_name())),
    }
}

/// net::resolve(hostname: String) -> Result<String, Error>
pub fn builtin_net_resolve(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let hostname = args[0].as_string()?;
    let addr_str = format!("{}:0", hostname);

    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                Ok(Value::Ok(Box::new(Value::String(addr.ip().to_string()))))
            } else {
                Ok(Value::Err(Box::new(Value::String(
                    "no addresses found".to_string(),
                ))))
            }
        }
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}
