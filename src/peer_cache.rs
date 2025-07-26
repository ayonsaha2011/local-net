use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::database;

// Single shared cache for all peer operations
static PEER_CACHE: std::sync::OnceLock<Arc<Mutex<HashMap<String, database::Peer>>>> = std::sync::OnceLock::new();

fn get_cache() -> &'static Arc<Mutex<HashMap<String, database::Peer>>> {
    PEER_CACHE.get_or_init(|| {
        println!("🔧 CACHE: Initializing peer cache");
        Arc::new(Mutex::new(HashMap::new()))
    })
}

pub fn add_peer(peer: database::Peer) {
    println!("🔄 CACHE: Adding peer to cache: {} ({}) at {}", peer.name, peer.id, peer.ip_address);
    
    let cache = get_cache();
    cache.lock().unwrap().insert(peer.id.clone(), peer);
    
    let cache_size = cache.lock().unwrap().len();
    println!("🔄 CACHE: Cache now has {} peers", cache_size);
}

pub fn remove_peer(peer_id: &str) {
    println!("🔄 CACHE: Removing peer from cache: {}", peer_id);
    
    let cache = get_cache();
    cache.lock().unwrap().remove(peer_id);
    
    let cache_size = cache.lock().unwrap().len();
    println!("🔄 CACHE: Cache now has {} peers", cache_size);
}

pub fn get_all_peers() -> Vec<database::Peer> {
    let cache = get_cache();
    let peers = cache.lock().unwrap().values().cloned().collect::<Vec<_>>();
    println!("🔍 CACHE: Returning {} peers from cache", peers.len());
    for peer in &peers {
        println!("   - {} ({}) at {} [{}]", peer.name, peer.id, peer.ip_address, 
                if peer.is_online { "online" } else { "offline" });
    }
    peers
}

pub fn get_cache_size() -> usize {
    let cache = get_cache();
    cache.lock().unwrap().len()
}