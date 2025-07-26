use dioxus::prelude::*;
use crate::{Route, database};

use std::future::Future;

#[component]
pub fn PeerList() -> Element {
    let mut show_dropdown = use_signal(|| false);
    let mut peers = use_signal(Vec::new);
    let mut search_query = use_signal(String::new);

    // Load peers from database and network discovery
    use_effect(move || {
        let mut all_peers = Vec::new();
        
        // Load from database first
        #[cfg(feature = "desktop")]
        if let Ok(db_peers) = database::get_peers() {
            all_peers.extend(db_peers);
        }
        
        // Add discovered peers from network
        let discovered_peers = crate::network_interface::discover_peers();
        for peer in discovered_peers {
            if !all_peers.iter().any(|p| p.id == peer.id) {
                all_peers.push(peer);
            }
        }
        
        peers.set(all_peers);
    });
    
    // Auto-refresh peers every 5 seconds
    use_effect(move || {
        let peers_setter = peers.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                
                let mut all_peers = Vec::new();
                
                // Load from database
                #[cfg(feature = "desktop")]
                if let Ok(db_peers) = database::get_peers() {
                    all_peers.extend(db_peers);
                }
                
                // Add discovered peers
                let discovered_peers = crate::network_interface::discover_peers();
                for peer in discovered_peers {
                    if !all_peers.iter().any(|p| p.id == peer.id) {
                        all_peers.push(peer);
                    }
                }
                
                peers_setter.set(all_peers);
            }
        });
    });

    let filtered_peers = peers.read().iter()
        .filter(|peer| {
            let query = search_query.read().to_lowercase();
            query.is_empty() || peer.name.to_lowercase().contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        div {
            class: "glass-container animate-slide-in",
            style: "
                display: flex; 
                flex-direction: column; 
                height: 100vh; 
                border-radius: 0;
                border: none;
                border-right: 1px solid var(--glass-border);
            ",
            
            // Header with search and menu
            div {
                style: "
                    padding: 20px; 
                    border-bottom: 1px solid var(--glass-border);
                    position: relative;
                ",
                div {
                    style: "
                        display: flex; 
                        justify-content: space-between; 
                        align-items: center;
                        margin-bottom: 16px;
                    ",
                    h1 { 
                        style: "
                            font-size: 24px; 
                            font-weight: 700; 
                            color: var(--text-primary);
                            margin: 0;
                        ",
                        "Conversations" 
                    }
                    div {
                        style: "position: relative;",
                        button {
                            class: "glass-button",
                            style: "
                                width: 40px; 
                                height: 40px; 
                                border-radius: 50%; 
                                display: flex; 
                                align-items: center; 
                                justify-content: center;
                                padding: 0;
                                font-size: 18px;
                                font-weight: bold;
                            ",
                            onclick: move |_| show_dropdown.set(!show_dropdown()),
                            "⋯"
                        }
                        if show_dropdown() {
                            div {
                                class: "glass-container animate-slide-in",
                                style: "
                                    position: absolute; 
                                    top: 100%; 
                                    right: 0; 
                                    border-radius: var(--border-radius);
                                    padding: 8px; 
                                    z-index: 1000; 
                                    display: flex; 
                                    flex-direction: column; 
                                    gap: 4px;
                                    min-width: 120px;
                                    margin-top: 8px;
                                ",
                                Link {
                                    to: Route::Profile {},
                                    class: "glass-button",
                                    style: "
                                        padding: 12px 16px; 
                                        text-decoration: none; 
                                        border-radius: 8px;
                                        display: flex;
                                        align-items: center;
                                        gap: 8px;
                                    ",
                                    "👤 Profile"
                                }
                                Link {
                                    to: Route::Settings {},
                                    class: "glass-button",
                                    style: "
                                        padding: 12px 16px; 
                                        text-decoration: none; 
                                        border-radius: 8px;
                                        display: flex;
                                        align-items: center;
                                        gap: 8px;
                                    ",
                                    "⚙️ Settings"
                                }
                            }
                        }
                    }
                }
                
                // Search input
                input {
                    class: "glass-input",
                    r#type: "text",
                    placeholder: "Search conversations...",
                    style: "
                        width: 100%; 
                        font-size: 16px;
                    ",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                }
            }

            // Peers list
            div {
                style: "
                    flex: 1; 
                    overflow-y: auto; 
                    padding: 0 20px 20px 20px;
                ",
                if filtered_peers.is_empty() {
                    div {
                        style: "
                            display: flex; 
                            flex-direction: column; 
                            align-items: center; 
                            justify-content: center; 
                            height: 200px; 
                            color: var(--text-secondary);
                            text-align: center;
                        ",
                        div {
                            style: "font-size: 48px; margin-bottom: 16px;",
                            "💬"
                        }
                        div {
                            style: "font-size: 18px; font-weight: 500; margin-bottom: 8px;",
                            "No conversations yet"
                        }
                        div {
                            style: "font-size: 14px;",
                            "Start chatting with peers on your network"
                        }
                    }
                } else {
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        for peer in filtered_peers {
                            PeerListItem {
                                key: "{peer.id}",
                                peer: peer.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PeerListItem(peer: database::Peer) -> Element {
    let is_online = peer.is_online;
    let mut last_message_text = use_signal(|| "No messages yet".to_string());
    
    // Load last message for this peer
    let peer_id_clone = peer.id.clone();
    use_effect(move || {
        #[cfg(feature = "desktop")]
        {
            if let Ok(Some(message)) = database::get_last_message_for_peer(&peer_id_clone) {
                let preview = if message.sender_id == "self" {
                    format!("You: {}", message.content)
                } else {
                    message.content
                };
                last_message_text.set(preview);
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            // Web version - no mock messages, will be populated by real chat
            last_message_text.set("No messages yet".to_string());
        }
    });
    
    rsx! {
        Link {
            to: Route::ChatView { peer_id: peer.id.clone() },
            style: "text-decoration: none;",
            div {
                class: "glass-container animate-slide-in",
                style: "
                    display: flex; 
                    align-items: center; 
                    gap: 16px; 
                    padding: 16px; 
                    border-radius: var(--border-radius);
                    cursor: pointer; 
                    transition: all 0.2s ease;
                    margin-bottom: 4px;
                ",
                onmouseenter: |_| {},
                onmouseleave: |_| {},
                
                // Avatar with online indicator
                div {
                    style: "position: relative;",
                    div {
                        style: "
                            width: 56px; 
                            height: 56px; 
                            border-radius: 50%; 
                            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            color: white;
                            font-weight: 600;
                            font-size: 20px;
                        ",
                        "{peer.name.chars().next().unwrap_or('?').to_uppercase()}"
                    }
                    if is_online {
                        div {
                            style: "
                                position: absolute; 
                                bottom: 2px; 
                                right: 2px; 
                                width: 16px; 
                                height: 16px; 
                                border-radius: 50%; 
                                background: var(--success-color);
                                border: 2px solid var(--glass-bg);
                            ",
                        }
                    }
                }
                
                // Chat info
                div {
                    style: "flex: 1; min-width: 0;",
                    div {
                        style: "
                            display: flex; 
                            justify-content: space-between; 
                            align-items: baseline;
                            margin-bottom: 4px;
                        ",
                        h3 {
                            style: "
                                font-size: 16px; 
                                font-weight: 600; 
                                color: var(--text-primary);
                                margin: 0;
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                            ",
                            "{peer.name}"
                        }
                        span {
                            style: "
                                font-size: 12px; 
                                color: var(--text-secondary);
                                white-space: nowrap;
                            ",
                            if is_online { "online" } else { "offline" }
                        }
                    }
                    div {
                        style: "
                            font-size: 14px; 
                            color: var(--text-secondary);
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
                        ",
                        "{last_message_text.read()}"
                    }
                    div {
                        style: "
                            font-size: 12px; 
                            color: var(--text-secondary);
                            margin-top: 2px;
                        ",
                        "📶 {peer.ip_address}"
                    }
                }
            }
        }
    }
}
