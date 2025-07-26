use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::database;

// Simple synchronous interface for the UI to interact with networking
pub struct NetworkInterface {
    pub discovered_peers: Arc<Mutex<Vec<database::Peer>>>,
    pub sent_messages: Arc<Mutex<Vec<database::Message>>>,
}

impl NetworkInterface {
    pub fn new() -> Self {
        Self {
            discovered_peers: Arc::new(Mutex::new(Vec::new())),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn get_discovered_peers(&self) -> Vec<database::Peer> {
        self.discovered_peers.lock().unwrap().clone()
    }
    
    pub fn add_discovered_peer(&self, peer: database::Peer) {
        let mut peers = self.discovered_peers.lock().unwrap();
        if !peers.iter().any(|p| p.id == peer.id) {
            peers.push(peer);
        }
    }
    
    pub fn send_message(&self, receiver_id: &str, content: &str) -> Result<(), String> {
        // Create message
        let message = database::Message {
            id: None,
            sender_id: "self".to_string(),
            receiver_id: receiver_id.to_string(),
            content: content.to_string(),
            encrypted_content: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: "text".to_string(),
            reply_to: None,
            file_path: None,
            file_size: None,
            is_read: true,
        };
        
        // Store in sent messages
        self.sent_messages.lock().unwrap().push(message.clone());
        
        // Store in database for desktop
        #[cfg(feature = "desktop")]
        if let Err(e) = database::insert_message(&message) {
            return Err(format!("Failed to store message: {}", e));
        }
        
        // TODO: Send over network (placeholder for now)
        println!("📤 Sending message to {}: {}", receiver_id, content);
        
        Ok(())
    }
    
    pub fn simulate_discovery(&self) {
        // Add some simulated peers for testing
        let test_peers = vec![
            database::Peer {
                id: "peer-1".to_string(),
                name: "Alice's Computer".to_string(),
                avatar: None,
                last_seen: chrono::Utc::now().to_rfc3339(),
                is_online: true,
                ip_address: "192.168.1.101".to_string(),
            },
            database::Peer {
                id: "peer-2".to_string(),
                name: "Bob's Laptop".to_string(),
                avatar: None,
                last_seen: chrono::Utc::now().to_rfc3339(),
                is_online: true,
                ip_address: "192.168.1.102".to_string(),
            },
        ];
        
        let mut peers = self.discovered_peers.lock().unwrap();
        peers.extend(test_peers);
        println!("🔍 Simulated peer discovery complete");
    }
}

// Global network interface instance
static NETWORK_INTERFACE: std::sync::OnceLock<NetworkInterface> = std::sync::OnceLock::new();

pub fn get_network_interface() -> &'static NetworkInterface {
    NETWORK_INTERFACE.get_or_init(|| {
        let interface = NetworkInterface::new();
        // Start peer discovery simulation after a short delay
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if let Some(interface) = NETWORK_INTERFACE.get() {
                interface.simulate_discovery();
            }
        });
        interface
    })
}

// Public API functions for UI
pub fn discover_peers() -> Vec<database::Peer> {
    get_network_interface().get_discovered_peers()
}

pub fn send_chat_message(receiver_id: &str, content: &str) -> Result<(), String> {
    get_network_interface().send_message(receiver_id, content)
}