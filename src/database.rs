#[cfg(feature = "desktop")]
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "desktop"))]
pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub last_seen: String,
    pub is_online: bool,
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: Option<i64>,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub encrypted_content: Option<String>,
    pub timestamp: String,
    pub message_type: String, // text, file, image, etc.
    pub reply_to: Option<i64>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileTransfer {
    pub id: Option<i64>,
    pub name: String,
    pub size: i64,
    pub path: String,
    pub status: String, // pending, accepted, rejected, downloading, completed, failed
    pub progress: f64,
    pub sender_id: String,
    pub receiver_id: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub username: String,
    pub app_mode: String, // dark, light, system
    pub notifications: bool,
    pub download_path: String,
    pub auto_accept_files: bool,
    pub max_file_size: Option<i64>,
}

#[cfg(feature = "desktop")]
pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("local-net-chat.db")?;

    // Peers table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS peers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            avatar TEXT,
            last_seen TEXT NOT NULL,
            ip_address TEXT NOT NULL
        )",
        [],
    )?;

    // Add missing columns if they don't exist (for migration)
    let _ = conn.execute("ALTER TABLE peers ADD COLUMN is_online BOOLEAN DEFAULT FALSE", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN encrypted_content TEXT", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN message_type TEXT DEFAULT 'text'", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN reply_to INTEGER", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN file_path TEXT", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN file_size INTEGER", []);
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN is_read BOOLEAN DEFAULT FALSE", []);

    // Messages table - create basic version first
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender_id TEXT NOT NULL,
            receiver_id TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    // File transfers table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_transfers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            size INTEGER NOT NULL,
            path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            progress REAL DEFAULT 0.0,
            sender_id TEXT NOT NULL,
            receiver_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (sender_id) REFERENCES peers (id),
            FOREIGN KEY (receiver_id) REFERENCES peers (id)
        )",
        [],
    )?;

    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Insert default settings if they don't exist
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES 
        ('username', 'User'),
        ('app_mode', 'system'),
        ('notifications', 'true'),
        ('download_path', './downloads'),
        ('auto_accept_files', 'false'),
        ('max_file_size', 'null')",
        [],
    )?;

    // Database initialized without demo data - ready for real peer discovery

    Ok(conn)
}

#[cfg(feature = "desktop")]
pub fn get_connection() -> Result<Connection> {
    Connection::open("local-net-chat.db")
}

#[cfg(feature = "desktop")]
pub fn get_peers() -> Result<Vec<Peer>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT id, name, avatar, last_seen, is_online, ip_address FROM peers ORDER BY last_seen DESC")?;
    
    let peer_iter = stmt.query_map([], |row| {
        Ok(Peer {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar: row.get(2)?,
            last_seen: row.get(3)?,
            is_online: row.get(4)?,
            ip_address: row.get(5)?,
        })
    })?;

    let mut peers = Vec::new();
    for peer in peer_iter {
        peers.push(peer?);
    }
    Ok(peers)
}

#[cfg(feature = "desktop")]
pub fn get_last_message_for_peer(peer_id: &str) -> Result<Option<Message>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, sender_id, receiver_id, content, encrypted_content, timestamp, 
         message_type, reply_to, file_path, file_size, is_read 
         FROM messages 
         WHERE (sender_id = ?1 AND receiver_id = 'self') OR (sender_id = 'self' AND receiver_id = ?1)
         ORDER BY timestamp DESC 
         LIMIT 1"
    )?;
    
    let message_result = stmt.query_row([peer_id], |row| {
        Ok(Message {
            id: Some(row.get(0)?),
            sender_id: row.get(1)?,
            receiver_id: row.get(2)?,
            content: row.get(3)?,
            encrypted_content: row.get(4)?,
            timestamp: row.get(5)?,
            message_type: row.get(6)?,
            reply_to: row.get(7)?,
            file_path: row.get(8)?,
            file_size: row.get(9)?,
            is_read: row.get(10)?,
        })
    });

    match message_result {
        Ok(message) => Ok(Some(message)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(feature = "desktop")]
pub fn get_messages_for_peer(peer_id: &str) -> Result<Vec<Message>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, sender_id, receiver_id, content, encrypted_content, timestamp, 
         message_type, reply_to, file_path, file_size, is_read 
         FROM messages 
         WHERE (sender_id = ?1 OR receiver_id = ?1) 
         ORDER BY timestamp ASC"
    )?;
    
    let message_iter = stmt.query_map([peer_id], |row| {
        Ok(Message {
            id: Some(row.get(0)?),
            sender_id: row.get(1)?,
            receiver_id: row.get(2)?,
            content: row.get(3)?,
            encrypted_content: row.get(4)?,
            timestamp: row.get(5)?,
            message_type: row.get(6)?,
            reply_to: row.get(7)?,
            file_path: row.get(8)?,
            file_size: row.get(9)?,
            is_read: row.get(10)?,
        })
    })?;

    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message?);
    }
    Ok(messages)
}

#[cfg(feature = "desktop")]
pub fn insert_message(message: &Message) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO messages (sender_id, receiver_id, content, encrypted_content, timestamp, 
         message_type, reply_to, file_path, file_size, is_read) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        [
            &message.sender_id,
            &message.receiver_id, 
            &message.content,
            message.encrypted_content.as_deref().unwrap_or(""),
            &message.timestamp,
            &message.message_type,
            &message.reply_to.map(|id| id.to_string()).as_deref().unwrap_or(""),
            &message.file_path.as_deref().unwrap_or(""),
            &message.file_size.map(|s| s.to_string()).as_deref().unwrap_or(""),
            &message.is_read.to_string(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(feature = "desktop")]
pub fn get_settings() -> Result<Settings> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    
    let settings_iter = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut username = "User".to_string();
    let mut app_mode = "system".to_string();
    let mut notifications = true;
    let mut download_path = "./downloads".to_string();
    let mut auto_accept_files = false;
    let mut max_file_size = None;

    for setting in settings_iter {
        let (key, value) = setting?;
        match key.as_str() {
            "username" => username = value,
            "app_mode" => app_mode = value,
            "notifications" => notifications = value == "true",
            "download_path" => download_path = value,
            "auto_accept_files" => auto_accept_files = value == "true",
            "max_file_size" => {
                if value != "null" {
                    max_file_size = value.parse().ok();
                }
            }
            _ => {}
        }
    }

    Ok(Settings {
        username,
        app_mode,
        notifications,
        download_path,
        auto_accept_files,
        max_file_size,
    })
}

#[cfg(feature = "desktop")]
pub fn insert_file_transfer(transfer: &FileTransfer) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO file_transfers (name, size, path, status, progress, sender_id, receiver_id, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &transfer.name,
            &transfer.size.to_string(),
            &transfer.path,
            &transfer.status,
            &transfer.progress.to_string(),
            &transfer.sender_id,
            &transfer.receiver_id,
            &transfer.timestamp,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(feature = "desktop")]
pub fn update_file_transfer_status(id: i64, status: &str, progress: f64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE file_transfers SET status = ?1, progress = ?2 WHERE id = ?3",
        [status, &progress.to_string(), &id.to_string()],
    )?;
    Ok(())
}

#[cfg(feature = "desktop")]
pub fn get_file_transfers() -> Result<Vec<FileTransfer>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, size, path, status, progress, sender_id, receiver_id, timestamp 
         FROM file_transfers ORDER BY timestamp DESC"
    )?;
    
    let transfer_iter = stmt.query_map([], |row| {
        Ok(FileTransfer {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            size: row.get::<_, i64>(2)?,
            path: row.get(3)?,
            status: row.get(4)?,
            progress: row.get::<_, String>(5)?.parse().unwrap_or(0.0),
            sender_id: row.get(6)?,
            receiver_id: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut transfers = Vec::new();
    for transfer in transfer_iter {
        transfers.push(transfer?);
    }
    Ok(transfers)
}

#[cfg(feature = "desktop")]
pub fn get_file_transfer_by_id(transfer_id: &str) -> Result<Option<FileTransfer>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, size, path, status, progress, sender_id, receiver_id, timestamp 
         FROM file_transfers WHERE name = ?1 OR id = ?2"
    )?;
    
    let mut transfer_iter = stmt.query_map([transfer_id, transfer_id], |row| {
        Ok(FileTransfer {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            size: row.get::<_, i64>(2)?,
            path: row.get(3)?,
            status: row.get(4)?,
            progress: row.get::<_, String>(5)?.parse().unwrap_or(0.0),
            sender_id: row.get(6)?,
            receiver_id: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    if let Some(transfer) = transfer_iter.next() {
        Ok(Some(transfer?))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "desktop")]
pub fn save_settings(settings: &Settings) -> Result<()> {
    let conn = get_connection()?;
    
    // Update each setting
    let settings_data = vec![
        ("username", settings.username.clone()),
        ("app_mode", settings.app_mode.clone()),
        ("notifications", settings.notifications.to_string()),
        ("download_path", settings.download_path.clone()),
        ("auto_accept_files", settings.auto_accept_files.to_string()),
        ("max_file_size", 
            settings.max_file_size.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string())
        ),
    ];
    
    for (key, value) in settings_data {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [key, &value],
        )?;
    }
    
    Ok(())
}

// Web/Mobile implementations using browser storage
#[cfg(not(feature = "desktop"))]
mod web_storage {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    
    // In-memory storage for web version (could be enhanced with localStorage)
    static PEERS: Mutex<Option<Vec<Peer>>> = Mutex::new(None);
    static MESSAGES: Mutex<Option<Vec<Message>>> = Mutex::new(None);
    static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
    static FILE_TRANSFERS: Mutex<Option<Vec<FileTransfer>>> = Mutex::new(None);
    
    fn init_storage() {
        let mut peers = PEERS.lock().unwrap();
        if peers.is_none() {
            *peers = Some(Vec::new());
        }
        
        let mut messages = MESSAGES.lock().unwrap();
        if messages.is_none() {
            *messages = Some(Vec::new());
        }
        
        let mut settings = SETTINGS.lock().unwrap();
        if settings.is_none() {
            *settings = Some(Settings {
                username: "WebUser".to_string(),
                app_mode: "system".to_string(),
                notifications: true,
                download_path: "./downloads".to_string(),
                auto_accept_files: false,
                max_file_size: None,
            });
        }
        
        let mut transfers = FILE_TRANSFERS.lock().unwrap();
        if transfers.is_none() {
            *transfers = Some(Vec::new());
        }
    }
    
    pub fn get_peers() -> Result<Vec<Peer>> {
        init_storage();
        let peers = PEERS.lock().map_err(|e| e.to_string())?;
        Ok(peers.as_ref().unwrap().clone())
    }
    
    pub fn get_messages_for_peer(peer_id: &str) -> Result<Vec<Message>> {
        init_storage();
        let messages = MESSAGES.lock().map_err(|e| e.to_string())?;
        let filtered: Vec<Message> = messages.as_ref().unwrap()
            .iter()
            .filter(|m| m.sender_id == peer_id || m.receiver_id == peer_id)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    pub fn insert_message(message: &Message) -> Result<i64> {
        init_storage();
        let mut messages = MESSAGES.lock().map_err(|e| e.to_string())?;
        let mut new_message = message.clone();
        new_message.id = Some(messages.as_ref().unwrap().len() as i64 + 1);
        messages.as_mut().unwrap().push(new_message);
        Ok(messages.as_ref().unwrap().len() as i64)
    }
    
    pub fn get_settings() -> Result<Settings> {
        init_storage();
        let settings = SETTINGS.lock().map_err(|e| e.to_string())?;
        Ok(settings.as_ref().unwrap().clone())
    }
    
    pub fn save_settings(new_settings: &Settings) -> Result<()> {
        init_storage();
        let mut settings = SETTINGS.lock().map_err(|e| e.to_string())?;
        *settings.as_mut().unwrap() = new_settings.clone();
        Ok(())
    }
    
    pub fn insert_file_transfer(transfer: &FileTransfer) -> Result<i64> {
        init_storage();
        let mut transfers = FILE_TRANSFERS.lock().map_err(|e| e.to_string())?;
        let mut new_transfer = transfer.clone();
        new_transfer.id = Some(transfers.as_ref().unwrap().len() as i64 + 1);
        transfers.as_mut().unwrap().push(new_transfer);
        Ok(transfers.as_ref().unwrap().len() as i64)
    }
    
    pub fn get_file_transfers() -> Result<Vec<FileTransfer>> {
        init_storage();
        let transfers = FILE_TRANSFERS.lock().map_err(|e| e.to_string())?;
        Ok(transfers.as_ref().unwrap().clone())
    }
    
    pub fn get_last_message_for_peer(peer_id: &str) -> Result<Option<Message>> {
        let messages = get_messages_for_peer(peer_id)?;
        Ok(messages.last().cloned())
    }
    
    pub fn update_file_transfer_status(_id: i64, _status: &str, _progress: f64) -> Result<()> {
        // TODO: Implement file transfer status updates
        Ok(())
    }
    
    pub fn get_file_transfer_by_id(_transfer_id: &str) -> Result<Option<FileTransfer>> {
        // TODO: Implement file transfer lookup
        Ok(None)
    }
}

// Export web functions when not on desktop
#[cfg(not(feature = "desktop"))]
pub use web_storage::*;
