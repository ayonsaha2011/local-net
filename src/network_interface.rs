use std::sync::{Arc, Mutex};
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
        
        // Send over network using the network manager
        #[cfg(feature = "desktop")]
        {
            let receiver_id = receiver_id.to_string();
            let content = content.to_string();
            tokio::spawn(async move {
                if let Err(e) = crate::network::send_chat_message(&receiver_id, &content).await {
                    eprintln!("Failed to send message over network: {}", e);
                }
            });
        }
        #[cfg(not(feature = "desktop"))]
        {
            println!("📤 Web message sent to {}: {}", receiver_id, content);
        }
        
        Ok(())
    }
    
    pub async fn update_discovered_peers(&self) {
        // Get real discovered peers from the network manager
        #[cfg(feature = "desktop")]
        {
            let peers = crate::network::get_discovered_peers().await;
            let mut discovered = self.discovered_peers.lock().unwrap();
            *discovered = peers;
        }
        #[cfg(not(feature = "desktop"))]
        {
            // For web, keep existing mock peers
            println!("📱 Web peers already simulated");
        }
    }
}

// Global network interface instance
static NETWORK_INTERFACE: std::sync::OnceLock<NetworkInterface> = std::sync::OnceLock::new();

pub fn get_network_interface() -> &'static NetworkInterface {
    NETWORK_INTERFACE.get_or_init(|| {
        let interface = NetworkInterface::new();
        // Start periodic peer list updates
        #[cfg(feature = "desktop")]
        {
            tokio::spawn(async {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    if let Some(interface) = NETWORK_INTERFACE.get() {
                        interface.update_discovered_peers().await;
                    }
                }
            });
        }
        #[cfg(not(feature = "desktop"))]
        {
            // For web, simulate some peers immediately
            let test_peers = vec![
                database::Peer {
                    id: "web-peer-1".to_string(),
                    name: "Mobile Device".to_string(),
                    avatar: None,
                    last_seen: chrono::Utc::now().to_rfc3339(),
                    is_online: true,
                    ip_address: "192.168.1.100".to_string(),
                },
                database::Peer {
                    id: "web-peer-2".to_string(),
                    name: "Tablet".to_string(),
                    avatar: None,
                    last_seen: chrono::Utc::now().to_rfc3339(),
                    is_online: true,
                    ip_address: "192.168.1.101".to_string(),
                },
            ];
            
            let mut peers = interface.discovered_peers.lock().unwrap();
            peers.extend(test_peers);
        }
        interface
    })
}

// Public API functions for UI  
pub fn discover_peers() -> Vec<database::Peer> {
    println!("🔍 UI: discover_peers() called");
    
    // Use the new mDNS network manager instead of the old interface
    #[cfg(feature = "desktop")]
    {
        let peers = crate::peer_cache::get_all_peers();
        println!("🔍 UI: Got {} peers from shared peer cache", peers.len());
        peers
    }
    
    #[cfg(not(feature = "desktop"))]
    {
        // For web, use the old interface simulation
        let peers = get_network_interface().get_discovered_peers();
        println!("🔍 UI: Got {} peers from web simulation", peers.len());
        peers
    }
}

// Functions to manage the peer cache (called by the network manager)
#[cfg(feature = "desktop")]
pub fn update_peer_cache(peer: crate::database::Peer) {
    crate::peer_cache::add_peer(peer);
}

#[cfg(feature = "desktop")]
pub fn remove_peer_from_cache(peer_id: &str) {
    crate::peer_cache::remove_peer(peer_id);
}

pub fn send_chat_message(receiver_id: &str, content: &str) -> Result<(), String> {
    get_network_interface().send_message(receiver_id, content)
}

pub fn send_file(_receiver_id: &str, _file_path: &str) -> Result<(), String> {
    #[cfg(feature = "desktop")]
    {
        // Get file info
        let file_metadata = std::fs::metadata(_file_path).map_err(|e| e.to_string())?;
        let file_size = file_metadata.len();
        let filename = std::path::Path::new(_file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Create file transfer record
        let transfer = database::FileTransfer {
            id: None,
            name: filename.clone(),
            size: file_size as i64,
            path: _file_path.to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            sender_id: "self".to_string(),
            receiver_id: _receiver_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Store in database
        if let Err(e) = database::insert_file_transfer(&transfer) {
            return Err(format!("Failed to store file transfer: {}", e));
        }
        
        // Send file transfer request over network
        let receiver_id = _receiver_id.to_string();
        let filename_clone = filename.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::network::send_file_transfer_request(&receiver_id, &filename_clone, file_size).await {
                eprintln!("Failed to send file transfer request: {}", e);
            }
        });
        
        println!("📤 File transfer request sent: {} ({} bytes)", filename, file_size);
        Ok(())
    }
    #[cfg(not(feature = "desktop"))]
    {
        println!("📎 Web file sending simulated for {}", _receiver_id);
        Ok(())
    }
}