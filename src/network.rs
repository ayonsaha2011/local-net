use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, connect_async, WebSocketStream, MaybeTlsStream};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use crate::database;

#[cfg(feature = "desktop")]
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

const SERVICE_TYPE: &str = "_local-net-chat._tcp.local.";
const SERVICE_PORT: u16 = 8081;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Chat messages
    ChatMessage {
        id: String,
        sender_id: String,
        receiver_id: String,
        content: String,
        timestamp: String,
        message_type: String,
    },
    
    // File sharing
    FileTransferRequest {
        id: String,
        sender_id: String,
        receiver_id: String,
        filename: String,
        file_size: u64,
        timestamp: String,
    },
    FileTransferResponse {
        id: String,
        accepted: bool,
        port: Option<u16>,
    },
    FileChunk {
        transfer_id: String,
        chunk_index: u32,
        total_chunks: u32,
        data: Vec<u8>,
    },
    
    // Status updates
    TypingIndicator {
        sender_id: String,
        receiver_id: String,
        is_typing: bool,
    },
    PeerOnline {
        peer_id: String,
        name: String,
        ip_address: String,
    },
    PeerOffline {
        peer_id: String,
    },
    
    // Peer discovery messages
    PeerDiscovery {
        peer_id: String,
        name: String,
        ip_address: String,
        port: u16,
    },
    PeerResponse {
        peer_id: String,
        name: String,
        ip_address: String,
        port: u16,
    },
}

pub struct NetworkManager {
    peer_id: String,
    username: String,
    port: u16,
    local_ip: String,
    peers: Arc<Mutex<HashMap<String, database::Peer>>>,
    message_sender: broadcast::Sender<Message>,
    #[cfg(feature = "desktop")]
    mdns_daemon: Option<ServiceDaemon>,
    service_name: String,
}

impl NetworkManager {
    pub fn new(username: String, port: u16) -> Self {
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = broadcast::channel(1000);
        let local_ip = Self::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
        let service_name = format!("local-net-chat-{}", peer_id);
        
        Self {
            peer_id,
            username,
            port,
            local_ip,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            #[cfg(feature = "desktop")]
            mdns_daemon: None,
            service_name,
        }
    }
    
    #[cfg(not(feature = "desktop"))]
    pub fn new_web(username: String) -> Self {
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = broadcast::channel(1000);
        let service_name = format!("local-net-chat-{}", peer_id);
        
        Self {
            peer_id,
            username,
            port: 8081,
            local_ip: "127.0.0.1".to_string(),
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            service_name,
        }
    }
    
    fn get_local_ip() -> Option<String> {
        #[cfg(feature = "desktop")]
        {
            // Try using local-ip-address crate first
            if let Ok(local_ip) = local_ip_address::local_ip() {
                let ip_str = local_ip.to_string();
                println!("🔍 mDNS: Detected local IP: {}", ip_str);
                return Some(ip_str);
            }
            
            // Fallback to the original method
            use std::net::UdpSocket;
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
                if socket.connect("8.8.8.8:80").is_ok() {
                    if let Ok(local_addr) = socket.local_addr() {
                        let ip_str = local_addr.ip().to_string();
                        println!("🔍 Fallback: Detected local IP: {}", ip_str);
                        return Some(ip_str);
                    }
                }
            }
        }
        
        println!("⚠️ Could not detect local IP, using fallback");
        None
    }
    
    #[cfg(feature = "desktop")]
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 Starting mDNS-based network services...");
        
        // Start WebSocket server for incoming connections
        let server_manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = server_manager.start_websocket_server().await {
                eprintln!("WebSocket server error: {}", e);
            }
        });
        
        // Start UDP peer discovery
        self.start_udp_discovery().await?;
        
        println!("✅ UDP network services started successfully");
        Ok(())
    }
    
    #[cfg(feature = "desktop")]
    async fn start_udp_discovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Starting UDP peer discovery...");
        
        // Start UDP listener
        let listener_manager = self.clone();
        tokio::spawn(async move {
            println!("🔍 Starting UDP listener task...");
            if let Err(e) = listener_manager.start_udp_listener().await {
                eprintln!("UDP listener error: {}", e);
            }
        });
        
        // Start UDP broadcaster
        let broadcaster_manager = self.clone();
        tokio::spawn(async move {
            println!("📡 Starting UDP broadcaster task...");
            broadcaster_manager.start_udp_broadcaster().await;
        });
        
        println!("✅ UDP discovery started");
        Ok(())
    }
    
    #[cfg(feature = "desktop")]
    async fn start_udp_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::net::UdpSocket;
        
        let socket = UdpSocket::bind("0.0.0.0:8082").await?;
        if let Ok(local_addr) = socket.local_addr() {
            println!("🔍 UDP listener bound to: {}", local_addr);
        } else {
            println!("🔍 UDP listener bound to port 8080");
        }
        
        println!("🔍 Listening for peer discovery messages from: {} ({})", self.username, self.peer_id);
        
        let mut buffer = [0; 1024];
        
        loop {
            match socket.recv_from(&mut buffer).await {
                Ok((len, addr)) => {
                    let data = &buffer[..len];
                    println!("🔍 ✅ Received {} bytes from {}", len, addr);
                    
                    match serde_json::from_slice::<Message>(data) {
                        Ok(message) => {
                            println!("🔍 📨 Parsed message: {:?}", message);
                            match message {
                            Message::PeerDiscovery { peer_id, name, ip_address, port: _ } => {
                                if peer_id != self.peer_id {
                                    println!("🆕 Discovered peer via UDP: {} at {}", name, ip_address);
                                    
                                    let peer = database::Peer {
                                        id: peer_id.clone(),
                                        name: name.clone(),
                                        avatar: None,
                                        last_seen: Utc::now().to_rfc3339(),
                                        is_online: true,
                                        ip_address: ip_address.clone(),
                                    };
                                    
                                    // Store peer
                                    {
                                        let mut peers_map = self.peers.lock().await;
                                        peers_map.insert(peer_id.clone(), peer.clone());
                                    }
                                    
                                    // Update UI cache
                                    crate::peer_cache::add_peer(peer.clone());
                                    
                                    // Send response back
                                    let response = Message::PeerResponse {
                                        peer_id: self.peer_id.clone(),
                                        name: self.username.clone(),
                                        ip_address: self.local_ip.clone(),
                                        port: self.port,
                                    };
                                    
                                    if let Ok(response_data) = serde_json::to_vec(&response) {
                                        let _ = socket.send_to(&response_data, addr).await;
                                    }
                                }
                            }
                            Message::PeerResponse { peer_id, name, ip_address, port: _ } => {
                                if peer_id != self.peer_id {
                                    println!("📨 Received peer response via UDP: {} at {}", name, ip_address);
                                    
                                    let peer = database::Peer {
                                        id: peer_id.clone(),
                                        name: name.clone(),
                                        avatar: None,
                                        last_seen: Utc::now().to_rfc3339(),
                                        is_online: true,
                                        ip_address: ip_address.clone(),
                                    };
                                    
                                    // Store peer
                                    {
                                        let mut peers_map = self.peers.lock().await;
                                        peers_map.insert(peer_id.clone(), peer.clone());
                                    }
                                    
                                    // Update UI cache
                                    crate::peer_cache::add_peer(peer);
                                }
                            }
                                _ => {
                                    println!("🔍 📨 Received other message type");
                                }
                            }
                        },
                        Err(e) => {
                            println!("🔍 ❌ Failed to parse message from {}: {}", addr, e);
                            println!("🔍    Raw data: {:?}", std::str::from_utf8(data).unwrap_or("invalid UTF-8"));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP receive error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    #[cfg(feature = "desktop")]
    async fn start_udp_broadcaster(&self) {
        use tokio::net::UdpSocket;
        
        // Wait a bit before starting broadcasts
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => {
                if let Ok(local_addr) = s.local_addr() {
                    println!("📡 UDP broadcaster bound to: {}", local_addr);
                } else {
                    println!("📡 UDP broadcaster bound to unknown local address");
                }
                s
            },
            Err(e) => {
                eprintln!("❌ Failed to bind UDP broadcaster: {}", e);
                return;
            }
        };
        
        if let Err(e) = socket.set_broadcast(true) {
            eprintln!("❌ Failed to set broadcast: {}", e);
            return;
        }
        
        println!("📡 UDP broadcaster started successfully");
        println!("📡 Will broadcast as: {} ({}) at {}", self.username, self.peer_id, self.local_ip);
        
        loop {
            let discovery_message = Message::PeerDiscovery {
                peer_id: self.peer_id.clone(),
                name: self.username.clone(),
                ip_address: self.local_ip.clone(),
                port: self.port,
            };
            
            match serde_json::to_vec(&discovery_message) {
                Ok(data) => {
                    // Broadcast to local network
                    match socket.send_to(&data, "255.255.255.255:8082").await {
                        Ok(bytes_sent) => {
                            println!("📡 ✅ Broadcast sent: {} bytes to 255.255.255.255:8082", bytes_sent);
                            println!("📡    Message: {} at {} (peer_id: {})", self.username, self.local_ip, self.peer_id);
                        },
                        Err(e) => {
                            eprintln!("📡 ❌ Broadcast failed: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("📡 ❌ Failed to serialize discovery message: {}", e);
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await; // More frequent broadcasts for debugging
        }
    }
    
    #[cfg(not(feature = "desktop"))]
    pub async fn start_web(&self) -> Result<(), String> {
        println!("📱 Starting web network services...");
        self.simulate_web_peers().await;
        println!("✅ Web network services started");
        Ok(())
    }
    
    async fn start_websocket_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("🚀 WebSocket server listening on {}", addr);
        
        while let Ok((stream, addr)) = listener.accept().await {
            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.handle_connection(stream, addr).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    async fn handle_connection(&self, stream: TcpStream, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        println!("🔗 New connection from {}", addr);
        
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    if msg.is_text() {
                        if let Ok(message) = serde_json::from_str::<Message>(&msg.to_text()?) {
                            self.handle_message(message, &mut ws_sender).await?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_message(&self, message: Message, ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Message>) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            Message::ChatMessage { id, sender_id, receiver_id, content, timestamp, message_type } => {
                println!("💬 ✅ Received chat message from {}: {}", sender_id, content);
                
                // Store message in database
                let db_message = database::Message {
                    id: None,
                    sender_id: sender_id.clone(),
                    receiver_id: receiver_id.clone(),
                    content: content.clone(),
                    encrypted_content: None,
                    timestamp: timestamp.clone(),
                    message_type: message_type.clone(),
                    reply_to: None,
                    file_path: None,
                    file_size: None,
                    is_read: false,
                };
                
                #[cfg(feature = "desktop")]
                match database::insert_message(&db_message) {
                    Ok(_) => {
                        println!("💬 📥 Message stored in database successfully");
                    }
                    Err(e) => {
                        eprintln!("💬 ❌ Failed to store message: {}", e);
                    }
                }
                
                // Broadcast message to UI
                let _ = self.message_sender.send(Message::ChatMessage {
                    id, sender_id, receiver_id, content, timestamp, message_type
                });
                println!("💬 📡 Message broadcasted to UI");
            }
            
            Message::FileTransferRequest { id, sender_id: _, receiver_id: _, filename, file_size, timestamp: _ } => {
                println!("📁 File transfer request: {} ({} bytes)", filename, file_size);
                
                // Auto-accept for now (implement proper logic later)
                let response = Message::FileTransferResponse {
                    id: id.clone(),
                    accepted: true,
                    port: Some(self.port + 1000),
                };
                
                let response_text = serde_json::to_string(&response)?;
                ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(response_text)).await?;
            }
            
            Message::TypingIndicator { sender_id, receiver_id, is_typing } => {
                // Broadcast typing indicator
                let _ = self.message_sender.send(Message::TypingIndicator {
                    sender_id, receiver_id, is_typing
                });
            }
            
            Message::FileTransferResponse { id, accepted, port } => {
                println!("📁 File transfer response: {} (accepted: {})", id, accepted);
                // Handle file transfer response
            }
            
            Message::FileChunk { transfer_id, chunk_index, total_chunks, data } => {
                println!("📦 Received file chunk {}/{} for transfer {}", 
                        chunk_index + 1, total_chunks, transfer_id);
                // Handle file chunk
            }
            
            Message::PeerOnline { peer_id, name, ip_address } => {
                println!("🟢 Peer came online: {} at {}", name, ip_address);
                let peer = database::Peer {
                    id: peer_id.clone(),
                    name: name.clone(),
                    avatar: None,
                    last_seen: chrono::Utc::now().to_rfc3339(),
                    is_online: true,
                    ip_address: ip_address.clone(),
                };
                crate::peer_cache::add_peer(peer);
            }
            
            Message::PeerOffline { peer_id } => {
                println!("🔴 Peer went offline: {}", peer_id);
                crate::peer_cache::remove_peer(&peer_id);
            }
            
            _ => {
                println!("📨 Received message: {:?}", message);
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "desktop"))]
    async fn simulate_web_peers(&self) {
        // Add some mock peers for web version
        let mock_peers = vec![
            database::Peer {
                id: "web-peer-1".to_string(),
                name: "Mobile Device".to_string(),
                avatar: None,
                last_seen: Utc::now().to_rfc3339(),
                is_online: true,
                ip_address: "192.168.1.100".to_string(),
            },
            database::Peer {
                id: "web-peer-2".to_string(),
                name: "Tablet".to_string(),
                avatar: None,
                last_seen: Utc::now().to_rfc3339(),
                is_online: true,
                ip_address: "192.168.1.101".to_string(),
            },
        ];
        
        let mut peers = self.peers.lock().await;
        for peer in mock_peers {
            peers.insert(peer.id.clone(), peer.clone());
            
            // Broadcast peer online event
            let _ = self.message_sender.send(Message::PeerOnline {
                peer_id: peer.id,
                name: peer.name,
                ip_address: peer.ip_address,
            });
        }
        
        println!("📱 Simulated {} web peers", peers.len());
    }
    
    pub async fn send_message(&self, receiver_id: &str, content: &str) -> Result<(), String> {
        println!("💬 Sending message to {}: {}", receiver_id, content);
        
        let message = Message::ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender_id: self.peer_id.clone(),
            receiver_id: receiver_id.to_string(),
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            message_type: "text".to_string(),
        };
        
        // Find peer and send message
        let peers = self.peers.lock().await;
        println!("💬 Looking for peer {} in {} discovered peers", receiver_id, peers.len());
        
        if let Some(peer) = peers.get(receiver_id) {
            println!("💬 Found peer: {} at {}:{}", peer.name, peer.ip_address, self.port);
            match self.connect_to_peer(&peer.ip_address, self.port).await {
                Ok(mut stream) => {
                    let message_text = serde_json::to_string(&message).map_err(|e| e.to_string())?;
                    match stream.send(tokio_tungstenite::tungstenite::Message::Text(message_text)).await {
                        Ok(_) => {
                            println!("💬 ✅ Message sent successfully to {}", peer.name);
                        }
                        Err(e) => {
                            println!("💬 ❌ Failed to send message: {}", e);
                            return Err(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    println!("💬 ❌ Failed to connect to peer {}:{}: {}", peer.ip_address, self.port, e);
                    return Err(format!("Failed to connect to peer: {}", e));
                }
            }
        } else {
            println!("💬 ❌ Peer {} not found in discovered peers", receiver_id);
            for (id, peer) in peers.iter() {
                println!("   Available peer: {} -> {} at {}", id, peer.name, peer.ip_address);
            }
            return Err(format!("Peer {} not found", receiver_id));
        }
        
        // Store message locally
        let db_message = database::Message {
            id: None,
            sender_id: self.peer_id.clone(),
            receiver_id: receiver_id.to_string(),
            content: content.to_string(),
            encrypted_content: None,
            timestamp: Utc::now().to_rfc3339(),
            message_type: "text".to_string(),
            reply_to: None,
            file_path: None,
            file_size: None,
            is_read: true,
        };
        
        #[cfg(feature = "desktop")]
        if let Err(e) = database::insert_message(&db_message) {
            eprintln!("Failed to store sent message: {}", e);
        }
        
        Ok(())
    }
    
    async fn connect_to_peer(&self, ip: &str, port: u16) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
        let url = format!("ws://{}:{}", ip, port);
        let (ws_stream, _) = connect_async(&url).await.map_err(|e| e.to_string())?;
        Ok(ws_stream)
    }
    
    pub async fn get_discovered_peers(&self) -> Vec<database::Peer> {
        let peers = self.peers.lock().await.values().cloned().collect::<Vec<_>>();
        println!("🔍 NetworkManager: Returning {} discovered peers", peers.len());
        for peer in &peers {
            println!("   - {} ({}) at {}", peer.name, peer.id, peer.ip_address);
        }
        peers
    }
    
    pub async fn send_file_transfer_request(&self, receiver_id: &str, filename: &str, file_size: u64) -> Result<(), String> {
        let request_id = Uuid::new_v4().to_string();
        let message = Message::FileTransferRequest {
            id: request_id,
            sender_id: self.peer_id.clone(),
            receiver_id: receiver_id.to_string(),
            filename: filename.to_string(),
            file_size,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        // Find peer and send file transfer request
        let peers = self.peers.lock().await;
        if let Some(peer) = peers.get(receiver_id) {
            if let Ok(mut stream) = self.connect_to_peer(&peer.ip_address, self.port).await {
                let message_text = serde_json::to_string(&message).map_err(|e| e.to_string())?;
                stream.send(tokio_tungstenite::tungstenite::Message::Text(message_text)).await
                    .map_err(|e| e.to_string())?;
                println!("📤 File transfer request sent to {}: {}", receiver_id, filename);
                return Ok(());
            }
        }
        
        Err(format!("Peer {} not found or not reachable", receiver_id))
    }
    
    pub fn subscribe_to_messages(&self) -> broadcast::Receiver<Message> {
        self.message_sender.subscribe()
    }
}

impl Clone for NetworkManager {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id.clone(),
            username: self.username.clone(),
            port: self.port,
            local_ip: self.local_ip.clone(),
            peers: Arc::clone(&self.peers),
            message_sender: self.message_sender.clone(),
            #[cfg(feature = "desktop")]
            mdns_daemon: None, // Can't clone the daemon
            service_name: self.service_name.clone(),
        }
    }
}

// Global network manager instance
static NETWORK_MANAGER: tokio::sync::OnceCell<NetworkManager> = tokio::sync::OnceCell::const_new();

pub async fn start_network() {
    println!("🌐 Starting mDNS network services...");
    
    #[cfg(feature = "desktop")]
    {
        let username = get_username_from_settings();
        let port = 8081;
        
        let mut manager = NetworkManager::new(username, port);
        
        if let Err(e) = manager.start().await {
            eprintln!("Failed to start network manager: {}", e);
            return;
        }
        
        if let Err(_) = NETWORK_MANAGER.set(manager) {
            eprintln!("Network manager already initialized");
        }
        
        println!("✅ mDNS network services started successfully");
    }
    
    #[cfg(not(feature = "desktop"))]
    {
        let username = get_username_from_settings();
        let manager = NetworkManager::new_web(username);
        
        if let Err(e) = manager.start_web().await {
            eprintln!("Failed to start web network manager: {}", e);
            return;
        }
        
        if let Err(_) = NETWORK_MANAGER.set(manager) {
            eprintln!("Network manager already initialized");
        }
        
        println!("✅ Web network services started successfully");
    }
}

pub async fn get_network_manager() -> Option<&'static NetworkManager> {
    NETWORK_MANAGER.get()
}

pub async fn send_chat_message(receiver_id: &str, content: &str) -> Result<(), String> {
    if let Some(manager) = get_network_manager().await {
        manager.send_message(receiver_id, content).await
    } else {
        Err("Network manager not initialized".into())
    }
}

pub async fn get_discovered_peers() -> Vec<database::Peer> {
    if let Some(manager) = get_network_manager().await {
        manager.get_discovered_peers().await
    } else {
        Vec::new()
    }
}

pub async fn send_file_transfer_request(receiver_id: &str, filename: &str, file_size: u64) -> Result<(), String> {
    if let Some(manager) = get_network_manager().await {
        manager.send_file_transfer_request(receiver_id, filename, file_size).await
    } else {
        Err("Network manager not initialized".into())
    }
}

// Helper function to get username from settings
fn get_username_from_settings() -> String {
    #[cfg(feature = "desktop")]
    {
        if let Ok(settings) = database::get_settings() {
            return settings.username;
        }
    }
    "User".to_string()
}