use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

/// Port used for UDP broadcast chat.
const CHAT_PORT: u16 = 47832;
/// Maximum number of chat messages to keep in history.
const MAX_CHAT_MESSAGES: usize = 50;
/// Maximum message length.
const MAX_MESSAGE_LEN: usize = 500;
/// Maximum sender identity length accepted from the network.
const MAX_SENDER_LEN: usize = 200;

/// A single chat message transmitted over the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: u64, // Unix epoch seconds
}

/// The local chat state maintained by each game instance.
pub struct ChatState {
    /// This user's identity: user@hostname.
    pub identity: String,
    /// Current input buffer for composing messages.
    pub input_buffer: String,
    /// All received (and sent) chat messages.
    pub messages: VecDeque<ChatMessage>,
    /// Known users seen in chat (for tab-completion of pings).
    pub known_users: Vec<String>,
    /// Whether the chat panel is currently visible.
    pub chat_open: bool,
    /// Whether this user was pinged (triggers bell on next render).
    pub pending_ping: bool,
    /// Position in tab-completion cycle (None if not cycling).
    pub tab_index: Option<usize>,
    /// The partial text before tab-completion started (used to filter candidates).
    pub tab_prefix: Option<String>,
    /// Shared message queue filled by the receiver thread.
    pub incoming: Arc<Mutex<Vec<ChatMessage>>>,
    /// UDP socket for sending messages (None if networking failed).
    socket: Option<UdpSocket>,
}

impl ChatState {
    /// Create a new chat state and attempt to bind the UDP socket.
    pub fn new() -> Self {
        let identity = build_identity();
        let socket = bind_chat_socket();
        let incoming = Arc::new(Mutex::new(Vec::new()));

        // Start receiver thread if we got a socket
        if let Some(ref sock) = socket {
            let recv_sock = sock.try_clone().ok();
            if let Some(recv_sock) = recv_sock {
                let incoming_clone = Arc::clone(&incoming);
                let my_identity = identity.clone();
                std::thread::spawn(move || {
                    receiver_thread(recv_sock, incoming_clone, my_identity);
                });
            }
        }

        ChatState {
            identity,
            input_buffer: String::new(),
            messages: VecDeque::new(),
            known_users: Vec::new(),
            chat_open: false,
            pending_ping: false,
            tab_index: None,
            tab_prefix: None,
            incoming,
            socket,
        }
    }

    /// Poll for incoming messages from the receiver thread.
    pub fn poll_incoming(&mut self) {
        if let Ok(mut queue) = self.incoming.lock() {
            for msg in queue.drain(..) {
                // Track known users for autocomplete
                if !self.known_users.contains(&msg.sender) {
                    self.known_users.push(msg.sender.clone());
                }
                // Check if we got pinged
                let ping_target = format!("${}", self.identity);
                if msg.content.contains(&ping_target) {
                    self.pending_ping = true;
                }
                if self.messages.len() >= MAX_CHAT_MESSAGES {
                    self.messages.pop_front();
                }
                self.messages.push_back(msg);
            }
        }
    }

    /// Send the current input buffer as a chat message.
    pub fn send_message(&mut self) {
        let content = self.input_buffer.trim().to_string();
        if content.is_empty() || content.len() > MAX_MESSAGE_LEN {
            return;
        }

        let msg = ChatMessage {
            sender: self.identity.clone(),
            content: content.clone(),
            timestamp: current_epoch(),
        };

        // Add to our own message list
        if self.messages.len() >= MAX_CHAT_MESSAGES {
            self.messages.pop_front();
        }
        self.messages.push_back(msg.clone());

        // Broadcast over UDP
        if let Some(ref sock) = self.socket {
            if let Ok(data) = serde_json::to_vec(&msg) {
                let broadcast_addr = format!("255.255.255.255:{}", CHAT_PORT);
                let _ = sock.send_to(&data, &broadcast_addr);
            }
        }

        self.input_buffer.clear();
        self.tab_index = None;
        self.tab_prefix = None;
    }

    /// Tab-complete a ping ($user@host) in the input buffer.
    pub fn tab_complete(&mut self) {
        if self.known_users.is_empty() {
            return;
        }

        // Find the word being typed (starting with $)
        let (prefix, start_pos) = match self.tab_prefix.clone() {
            Some(p) => (p.clone(), self.find_ping_start()),
            None => {
                let pos = self.find_ping_start();
                let prefix = if pos < self.input_buffer.len() {
                    self.input_buffer[pos..].to_string()
                } else {
                    String::new()
                };
                self.tab_prefix = Some(prefix.clone());
                (prefix, pos)
            }
        };

        // Filter candidates matching the prefix (case insensitive)
        let search = prefix.trim_start_matches('$').to_lowercase();
        let candidates: Vec<&String> = self
            .known_users
            .iter()
            .filter(|u| {
                **u != self.identity && u.to_lowercase().contains(&search)
            })
            .collect();

        if candidates.is_empty() {
            return;
        }

        let idx = match self.tab_index {
            Some(i) => (i + 1) % candidates.len(),
            None => 0,
        };
        self.tab_index = Some(idx);

        // Replace the ping text in the input buffer
        let replacement = format!("${} ", candidates[idx]);
        self.input_buffer.truncate(start_pos);
        self.input_buffer.push_str(&replacement);
    }

    /// Find the start position of the current $ping word.
    fn find_ping_start(&self) -> usize {
        if let Some(pos) = self.input_buffer.rfind('$') {
            // Only if there's no space after $
            if !self.input_buffer[pos..].contains(' ') || self.tab_prefix.is_some() {
                return pos;
            }
        }
        self.input_buffer.len()
    }

    /// Reset tab completion state (call when user types a non-tab key).
    pub fn reset_tab(&mut self) {
        self.tab_index = None;
        self.tab_prefix = None;
    }

    /// Check if this message contains a ping for any known user.
    #[allow(dead_code)]
    pub fn message_contains_ping(content: &str) -> bool {
        content.contains('$')
    }

    /// Check if a specific user is pinged in the content.
    pub fn is_user_pinged(content: &str, identity: &str) -> bool {
        let ping = format!("${}", identity);
        content.contains(&ping)
    }

    /// Consume the pending ping flag and return whether a bell should be played.
    pub fn consume_ping(&mut self) -> bool {
        if self.pending_ping {
            self.pending_ping = false;
            true
        } else {
            false
        }
    }

    /// Check if networking is available.
    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }
}

/// Build the user identity string: user@hostname.
fn build_identity() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "player".to_string());
    let hostname = gethostname::gethostname()
        .to_string_lossy()
        .to_string();
    format!("{}@{}", user, hostname)
}

/// Try to bind a UDP socket for chat broadcast.
fn bind_chat_socket() -> Option<UdpSocket> {
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", CHAT_PORT)).ok()?;
    sock.set_broadcast(true).ok()?;
    // Set a read timeout so the receiver thread doesn't block indefinitely
    sock.set_read_timeout(Some(std::time::Duration::from_millis(200))).ok()?;
    Some(sock)
}

/// Background thread that listens for incoming chat messages.
fn receiver_thread(
    sock: UdpSocket,
    incoming: Arc<Mutex<Vec<ChatMessage>>>,
    my_identity: String,
) {
    let mut buf = [0u8; 8192];
    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, _addr)) => {
                if let Ok(msg) = serde_json::from_slice::<ChatMessage>(&buf[..n]) {
                    // Skip our own messages (we already added them locally)
                    if msg.sender == my_identity {
                        continue;
                    }
                    // Validate incoming message: reject excessively long content or sender
                    if msg.content.len() > MAX_MESSAGE_LEN || msg.sender.len() > MAX_SENDER_LEN {
                        continue;
                    }
                    if let Ok(mut queue) = incoming.lock() {
                        queue.push(msg);
                    }
                }
            }
            Err(ref e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::TimedOut =>
            {
                // Read timeout or would-block: just loop and try again
                continue;
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
    }
}

/// Get current unix epoch seconds.
fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_identity_format() {
        let id = build_identity();
        assert!(id.contains('@'), "Identity should contain @: {}", id);
    }

    #[test]
    fn test_is_user_pinged() {
        assert!(ChatState::is_user_pinged("hello $alice@host", "alice@host"));
        assert!(!ChatState::is_user_pinged("hello alice@host", "alice@host"));
        assert!(!ChatState::is_user_pinged("hello $bob@host", "alice@host"));
    }

    #[test]
    fn test_message_contains_ping() {
        assert!(ChatState::message_contains_ping("hey $user@pc"));
        assert!(!ChatState::message_contains_ping("hello world"));
    }

    #[test]
    fn test_chat_send_message() {
        let mut chat = ChatState {
            identity: "test@host".to_string(),
            input_buffer: "hello world".to_string(),
            messages: VecDeque::new(),
            known_users: Vec::new(),
            chat_open: true,
            pending_ping: false,
            tab_index: None,
            tab_prefix: None,
            incoming: Arc::new(Mutex::new(Vec::new())),
            socket: None, // No network in tests
        };
        chat.send_message();
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.messages[0].content, "hello world");
        assert_eq!(chat.messages[0].sender, "test@host");
        assert!(chat.input_buffer.is_empty());
    }

    #[test]
    fn test_chat_empty_message_not_sent() {
        let mut chat = ChatState {
            identity: "test@host".to_string(),
            input_buffer: "   ".to_string(),
            messages: VecDeque::new(),
            known_users: Vec::new(),
            chat_open: true,
            pending_ping: false,
            tab_index: None,
            tab_prefix: None,
            incoming: Arc::new(Mutex::new(Vec::new())),
            socket: None,
        };
        chat.send_message();
        assert_eq!(chat.messages.len(), 0);
    }

    #[test]
    fn test_poll_incoming_detects_ping() {
        let incoming = Arc::new(Mutex::new(Vec::new()));
        let mut chat = ChatState {
            identity: "me@host".to_string(),
            input_buffer: String::new(),
            messages: VecDeque::new(),
            known_users: Vec::new(),
            chat_open: true,
            pending_ping: false,
            tab_index: None,
            tab_prefix: None,
            incoming: Arc::clone(&incoming),
            socket: None,
        };
        // Simulate an incoming message that pings us
        {
            let mut q = incoming.lock().unwrap();
            q.push(ChatMessage {
                sender: "other@host".to_string(),
                content: "hey $me@host check this out".to_string(),
                timestamp: 12345,
            });
        }
        chat.poll_incoming();
        assert!(chat.pending_ping);
        assert_eq!(chat.messages.len(), 1);
        assert!(chat.known_users.contains(&"other@host".to_string()));
    }

    #[test]
    fn test_consume_ping() {
        let mut chat = ChatState {
            identity: "test@host".to_string(),
            input_buffer: String::new(),
            messages: VecDeque::new(),
            known_users: Vec::new(),
            chat_open: false,
            pending_ping: true,
            tab_index: None,
            tab_prefix: None,
            incoming: Arc::new(Mutex::new(Vec::new())),
            socket: None,
        };
        assert!(chat.consume_ping());
        assert!(!chat.consume_ping()); // Should be consumed
    }
}
