//! Unix socket-based IPC implementation for daemon communication.

use super::{DaemonIpc, IpcError, IpcRequest, IpcResponse, DEFAULT_TIMEOUT};
use interprocess::local_socket::LocalSocketStream;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Unix socket IPC client implementation
pub struct UnixSocketIpc {
    socket_path: PathBuf,
    timeout: Duration,
    stream: Option<LocalSocketStream>,
}

impl UnixSocketIpc {
    /// Create a new Unix socket IPC client
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            timeout: DEFAULT_TIMEOUT,
            stream: None,
        }
    }

    /// Create a new Unix socket IPC client with custom timeout
    pub fn with_timeout(socket_path: PathBuf, timeout: Duration) -> Self {
        Self {
            socket_path,
            timeout,
            stream: None,
        }
    }

    /// Connect to the daemon socket
    fn connect(&mut self) -> Result<(), IpcError> {
        // Check if socket file exists
        if !self.socket_path.exists() {
            return Err(IpcError::SocketNotFound(
                self.socket_path.display().to_string(),
            ));
        }

        // Get the socket name based on platform support
        let name = self.socket_path.to_string_lossy();

        // Connect to the socket
        let stream = LocalSocketStream::connect(name.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                IpcError::ConnectionRefused
            } else {
                IpcError::IoError(e)
            }
        })?;

        // Note: LocalSocketStream doesn't support set_timeout directly
        // We'll implement timeout using a manual timeout check
        self.stream = Some(stream);
        Ok(())
    }

    /// Send a request and receive a response
    fn send_and_receive(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        // Ensure we're connected
        if self.stream.is_none() {
            self.connect()?;
        }

        let start_time = Instant::now();
        let stream = self.stream.as_mut().unwrap();

        // Serialize request to JSON
        let json =
            serde_json::to_string(request).map_err(|e| IpcError::SerializeError(e.to_string()))?;

        // Send request (newline-delimited)
        stream
            .write_all(json.as_bytes())
            .map_err(IpcError::IoError)?;
        stream.write_all(b"\n").map_err(IpcError::IoError)?;
        stream.flush().map_err(IpcError::IoError)?;

        // Check timeout before reading
        if start_time.elapsed() >= self.timeout {
            return Err(IpcError::Timeout(self.timeout));
        }

        // Read response (newline-delimited)
        let mut response_line = String::new();
        let mut reader = BufReader::new(stream);

        reader.read_line(&mut response_line).map_err(|e| {
            if e.kind() == std::io::ErrorKind::WouldBlock || start_time.elapsed() >= self.timeout {
                IpcError::Timeout(self.timeout)
            } else {
                IpcError::IoError(e)
            }
        })?;

        // Deserialize response
        let response: IpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| IpcError::DeserializeError(e.to_string()))?;

        Ok(response)
    }
}

impl DaemonIpc for UnixSocketIpc {
    fn send_request(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        self.send_and_receive(request)
    }

    fn receive_response(&mut self) -> Result<IpcResponse, IpcError> {
        // This is used by the server-side (daemon) to receive requests
        // For client-side, we use send_and_receive
        Err(IpcError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "receive_response is only for server-side use",
        )))
    }
}

impl Drop for UnixSocketIpc {
    fn drop(&mut self) {
        // Stream will be automatically closed when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::LocalSocketListener;
    use std::io::BufRead;
    use std::thread;
    use tempfile::TempDir;

    fn setup_test_socket() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test.sock");
        (temp_dir, socket_path)
    }

    #[test]
    fn test_socket_not_found() {
        let (_temp_dir, socket_path) = setup_test_socket();
        let mut client = UnixSocketIpc::new(socket_path.clone());

        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(matches!(result, Err(IpcError::SocketNotFound(_))));
    }

    #[test]
    fn test_connect_and_roundtrip() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start a simple echo server in a background thread
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");

            // Accept one connection
            let mut conn = listener.accept().expect("Failed to accept connection");

            // Read request
            let mut reader = BufReader::new(&mut conn);
            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("Failed to read request");

            // Parse request
            let _request: IpcRequest =
                serde_json::from_str(&request_line).expect("Failed to parse request");

            // Send response
            let response = IpcResponse::Status {
                running: true,
                uptime_secs: 100,
                active_profile: Some("test".to_string()),
                device_count: 1,
            };
            let json = serde_json::to_string(&response).expect("Failed to serialize response");
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        });

        // Wait for server to start
        thread::sleep(Duration::from_millis(100));

        // Create client and send request
        let mut client = UnixSocketIpc::new(socket_path);
        let response = client
            .send_request(&IpcRequest::GetStatus)
            .expect("Request failed");

        match response {
            IpcResponse::Status {
                running,
                uptime_secs,
                active_profile,
                device_count,
            } => {
                assert!(running);
                assert_eq!(uptime_secs, 100);
                assert_eq!(active_profile, Some("test".to_string()));
                assert_eq!(device_count, 1);
            }
            _ => panic!("Unexpected response type"),
        }

        server_handle.join().unwrap();
    }

    #[test]
    fn test_timeout_handling() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start a server that never responds
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");

            // Accept connection but never respond
            let _conn = listener.accept().expect("Failed to accept connection");
            thread::sleep(Duration::from_secs(10)); // Sleep longer than timeout
        });

        // Wait for server to start
        thread::sleep(Duration::from_millis(100));

        // Create client with short timeout
        let mut client = UnixSocketIpc::with_timeout(socket_path, Duration::from_millis(100));
        let result = client.send_request(&IpcRequest::GetStatus);

        assert!(matches!(result, Err(IpcError::Timeout(_))));

        // Let server thread finish
        drop(server_handle);
    }

    #[test]
    fn test_custom_timeout() {
        let (_temp_dir, socket_path) = setup_test_socket();
        let client = UnixSocketIpc::with_timeout(socket_path, Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_request_serialization() {
        let request = IpcRequest::GetEventsTail { count: 50 };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("get_events_tail"));
        assert!(json.contains("50"));
    }
}
