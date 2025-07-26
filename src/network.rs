use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio_tungstenite::{accept_async, connect_async, WebSocketStream, MaybeTlsStream};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use crate::database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Peer discovery
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
}

pub struct NetworkManager {
    peer_id: String,
    username: String,
    port: u16,
    peers: Arc<Mutex<HashMap<String, database::Peer>>>,
    message_sender: broadcast::Sender<Message>,
    active_connections: Arc<Mutex<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl NetworkManager {
    pub fn new(username: String, port: u16) -> Self {
        let peer_id = Uuid::new_v4().to_string();
        let (message_sender, _) = broadcast::channel(1000);
        
        Self {
            peer_id,
            username,
            port,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            active_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start WebSocket server for incoming connections
        let server_manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = server_manager.start_websocket_server().await {
                eprintln!("WebSocket server error: {}", e);
            }
        });
        
        // Start UDP peer discovery
        let discovery_manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = discovery_manager.start_peer_discovery().await {
                eprintln!("Peer discovery error: {}", e);
            }
        });
        
        // Start periodic peer discovery broadcast
        let broadcast_manager = self.clone();
        tokio::spawn(async move {
            broadcast_manager.broadcast_peer_discovery().await;
        });
        
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
    
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
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
                if let Err(e) = database::insert_message(&db_message) {
                    eprintln!("Failed to store message: {}", e);
                }
                
                // Broadcast message to UI
                let _ = self.message_sender.send(Message::ChatMessage {
                    id, sender_id, receiver_id, content, timestamp, message_type
                });
            }
            
            Message::PeerDiscovery { peer_id, name, ip_address, port } => {
                // Add/update peer
                let peer = database::Peer {
                    id: peer_id.clone(),
                    name: name.clone(),
                    avatar: None,
                    last_seen: Utc::now().to_rfc3339(),
                    is_online: true,
                    ip_address: ip_address.clone(),
                };
                
                self.peers.lock().await.insert(peer_id.clone(), peer);
                
                // Send response
                let response = Message::PeerResponse {
                    peer_id: self.peer_id.clone(),
                    name: self.username.clone(),
                    ip_address: "127.0.0.1".to_string(), // TODO: Get actual IP
                    port: self.port,
                };
                
                let response_text = serde_json::to_string(&response)?;
                ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(response_text)).await?;
                
                // Broadcast peer online event
                let _ = self.message_sender.send(Message::PeerOnline {
                    peer_id: peer_id.clone(),
                    name,
                    ip_address,
                });
            }
            
            Message::FileTransferRequest { id, sender_id, receiver_id, filename, file_size, timestamp } => {
                // Handle file transfer request
                println!("📁 File transfer request: {} ({} bytes)", filename, file_size);
                
                // For now, auto-accept files (TODO: Add user confirmation)
                let response = Message::FileTransferResponse {
                    id: id.clone(),
                    accepted: true,
                    port: Some(self.port + 1000), // Use different port for file transfer
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
            
            _ => {
                // Handle other message types
                println!("📨 Received message: {:?}", message);
            }
        }
        
        Ok(())
    }
    
    async fn start_peer_discovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:8080").await?;
        socket.set_broadcast(true)?;
        
        println!("🔍 UDP peer discovery listening on port 8080");
        
        let mut buffer = [0; 1024];
        
        loop {
            if let Ok((len, addr)) = socket.recv_from(&mut buffer).await {
                let data = &buffer[..len];
                
                if let Ok(message) = serde_json::from_slice::<Message>(data) {
                    match message {
                        Message::PeerDiscovery { peer_id, name, ip_address, port } => {
                            if peer_id != self.peer_id {
                                println!("🆕 Discovered peer: {} ({}:{})", name, ip_address, port);
                                
                                let peer = database::Peer {
                                    id: peer_id.clone(),
                                    name: name.clone(),
                                    avatar: None,
                                    last_seen: Utc::now().to_rfc3339(),
                                    is_online: true,
                                    ip_address: ip_address.clone(),
                                };
                                
                                self.peers.lock().await.insert(peer_id.clone(), peer);
                                
                                // Send response
                                let response = Message::PeerResponse {
                                    peer_id: self.peer_id.clone(),
                                    name: self.username.clone(),
                                    ip_address: "127.0.0.1".to_string(),
                                    port: self.port,
                                };
                                
                                let response_data = serde_json::to_vec(&response)?;
                                let _ = socket.send_to(&response_data, addr).await;
                                
                                // Broadcast peer discovery event
                                let _ = self.message_sender.send(Message::PeerOnline {
                                    peer_id,
                                    name,
                                    ip_address,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    async fn broadcast_peer_discovery(&self) {
        let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind UDP socket");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        
        let discovery_message = Message::PeerDiscovery {
            peer_id: self.peer_id.clone(),
            name: self.username.clone(),
            ip_address: "127.0.0.1".to_string(), // TODO: Get actual IP
            port: self.port,
        };
        
        loop {
            let data = serde_json::to_vec(&discovery_message).expect("Failed to serialize discovery message");
            let _ = socket.send_to(&data, "255.255.255.255:8080").await;
            
            println!("📡 Broadcasting peer discovery...");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }
    
    pub async fn send_message(&self, receiver_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Some(peer) = peers.get(receiver_id) {
            if let Ok(mut stream) = self.connect_to_peer(&peer.ip_address, self.port).await {
                let message_text = serde_json::to_string(&message)?;
                stream.send(tokio_tungstenite::tungstenite::Message::Text(message_text)).await?;
            }
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
    
    async fn connect_to_peer(&self, ip: &str, port: u16) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
        let url = format!("ws://{}:{}", ip, port);
        let (ws_stream, _) = connect_async(&url).await?;
        Ok(ws_stream)
    }
    
    pub async fn get_discovered_peers(&self) -> Vec<database::Peer> {
        self.peers.lock().await.values().cloned().collect()
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
            peers: Arc::clone(&self.peers),
            message_sender: self.message_sender.clone(),
            active_connections: Arc::clone(&self.active_connections),
        }
    }
}

// Global network manager instance
static NETWORK_MANAGER: tokio::sync::OnceCell<NetworkManager> = tokio::sync::OnceCell::const_new();

pub async fn start_network() {
    println!("🌐 Starting network services...");
    
    // Get username from settings or use default
    let username = "User".to_string(); // TODO: Get from settings
    let port = 8081;
    
    let manager = NetworkManager::new(username, port);
    
    if let Err(e) = manager.start().await {
        eprintln!("Failed to start network manager: {}", e);
        return;
    }
    
    if let Err(_) = NETWORK_MANAGER.set(manager) {
        eprintln!("Network manager already initialized");
    }
    
    println!("✅ Network services started successfully");
}

pub async fn get_network_manager() -> Option<&'static NetworkManager> {
    NETWORK_MANAGER.get()
}

pub async fn send_chat_message(receiver_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
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