use dioxus::prelude::*;
use dioxus::events::{KeyboardData};
use crate::{Route, database};

fn send_file(peer_id: &str) {
    #[cfg(feature = "desktop")]
    {
        if let Some(file_path) = select_file_to_send() {
            if let Err(e) = crate::network_interface::send_file(peer_id, &file_path) {
                eprintln!("Failed to send file: {}", e);
            }
        }
    }
    #[cfg(not(feature = "desktop"))]
    {
        // For web version, we need to use HTML file input
        // This is a placeholder - in a real web app, you'd trigger a file input element
        println!("📎 Opening file selector for web...");
        web_select_file(peer_id);
    }
}

#[cfg(not(feature = "desktop"))]
fn web_select_file(peer_id: &str) {
    // In a real web implementation, this would:
    // 1. Create or trigger an HTML file input element
    // 2. Handle the file selection event
    // 3. Read the file using FileReader API
    // 4. Send the file data
    
    // For now, simulate file selection
    println!("📱 Web file selector opened");
    
    // Simulate a file being selected
    let mock_file = crate::database::FileTransfer {
        id: None,
        name: "web_file.txt".to_string(),
        size: 1024,
        path: "mock://web_file.txt".to_string(),
        status: "pending".to_string(),
        progress: 0.0,
        sender_id: "self".to_string(),
        receiver_id: peer_id.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // Store in web storage
    if let Err(e) = crate::database::insert_file_transfer(&mock_file) {
        eprintln!("Failed to store mock file transfer: {}", e);
    } else {
        println!("📤 Mock file transfer created for web");
    }
}

#[cfg(feature = "desktop")]
fn select_file_to_send() -> Option<String> {
    use std::process::Command;
    
    // Try to use native file dialogs on different platforms
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("zenity")
            .args(&["--file-selection", "--title=Select File to Send"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(path);
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, we could use PowerShell file dialogs
        // For now, just use a placeholder
        println!("📁 Please enter file path manually (file dialog not implemented for Windows)");
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use osascript
        println!("📁 Please enter file path manually (file dialog not implemented for macOS)");
    }
    
    None
}

#[component]
pub fn ChatView(peer_id: String) -> Element {
    let mut messages = use_signal(Vec::new);
    let mut message_input = use_signal(String::new);
    let peer_id_for_effect = peer_id.clone();

    // Load messages from database or use mock data
    use_effect(move || {
        #[cfg(feature = "desktop")]
        {
            if let Ok(db_messages) = database::get_messages_for_peer(&peer_id_for_effect) {
                messages.set(db_messages);
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            // Web version starts with empty messages - will be populated by real chat
            messages.set(vec![]);
        }
    });

    let peer_id_for_send = peer_id.clone();
    let send_message = use_callback(move |_| {
        let content = message_input.read().clone();
        if !content.trim().is_empty() {
            // Store locally first
            let new_message = database::Message {
                id: None,
                sender_id: "self".to_string(),
                receiver_id: peer_id_for_send.clone(),
                content: content.clone(),
                encrypted_content: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                message_type: "text".to_string(),
                reply_to: None,
                file_path: None,
                file_size: None,
                is_read: true,
            };
            
            // Update UI immediately
            let mut current_messages = messages.read().clone();
            current_messages.push(new_message.clone());
            messages.set(current_messages);
            
            // Store in database if desktop
            #[cfg(feature = "desktop")]
            if let Err(e) = database::insert_message(&new_message) {
                eprintln!("Failed to store message: {}", e);
            }
            
            // Send over network interface
            if let Err(e) = crate::network_interface::send_chat_message(&peer_id_for_send, &content) {
                eprintln!("Failed to send message: {}", e);
            }
            
            message_input.set(String::new());
        }
    });

    rsx! {
        div {
            class: "glass-container animate-slide-in",
            style: "display: flex; flex-direction: column; height: 100vh; border-radius: 0; border: none;",
            
            // Chat header
            div {
                class: "glass-container",
                style: "padding: 16px 20px; border-bottom: 1px solid var(--glass-border); border-radius: 0; border: none; border-bottom: 1px solid var(--glass-border);",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    div {
                        style: "display: flex; align-items: center; gap: 12px;",
                        Link {
                            to: Route::PeerList {},
                            class: "glass-button",
                            style: "width: 40px; height: 40px; border-radius: 50%; display: flex; align-items: center; justify-content: center; padding: 0; font-size: 18px;",
                            "←"
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 12px;",
                            div {
                                style: "width: 40px; height: 40px; border-radius: 50%; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); display: flex; align-items: center; justify-content: center; color: white; font-weight: 600; font-size: 16px;",
                                "{peer_id.chars().next().unwrap_or('?').to_uppercase()}"
                            }
                            div {
                                h2 {
                                    style: "font-size: 18px; font-weight: 600; color: var(--text-primary); margin: 0;",
                                    "{peer_id}"
                                }
                                div {
                                    style: "font-size: 12px; color: var(--success-color);",
                                    "online"
                                }
                            }
                        }
                    }
                }
            }

            // Messages area
            div {
                style: "flex: 1; padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 12px;",
                for (i, message) in messages.read().iter().enumerate() {
                    MessageBubble {
                        key: "{i}",
                        message: message.clone(),
                    }
                }
            }

            // Input area
            div {
                class: "glass-container",
                style: "padding: 16px 20px; border-top: 1px solid var(--glass-border); border-radius: 0; border: none; border-top: 1px solid var(--glass-border);",
                div {
                    style: "display: flex; gap: 12px; align-items: flex-end;",
                    textarea {
                        class: "glass-input",
                        placeholder: "Type a message...",
                        style: "flex: 1; min-height: 40px; max-height: 100px; resize: none; font-size: 16px;",
                        value: "{message_input}",
                        oninput: move |evt| message_input.set(evt.value()),
                        onkeydown: {
                            let send_message = send_message.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == dioxus::events::Key::Enter && !evt.modifiers().shift() {
                                    evt.prevent_default();
                                    send_message(());
                                }
                            }
                        },
                    }
                    button {
                        class: "glass-button",
                        style: "width: 48px; height: 48px; border-radius: 50%; display: flex; align-items: center; justify-content: center; padding: 0; font-size: 18px; margin-right: 8px;",
                        onclick: move |_| {
                            send_file(&peer_id);
                        },
                        "📎"
                    }
                    button {
                        class: if message_input.read().trim().is_empty() { "glass-button" } else { "glass-button primary" },
                        style: "width: 48px; height: 48px; border-radius: 50%; display: flex; align-items: center; justify-content: center; padding: 0; font-size: 18px;",
                        onclick: {
                            let send_message = send_message.clone();
                            move |_| send_message(())
                        },
                        disabled: message_input.read().trim().is_empty(),
                        if message_input.read().trim().is_empty() { "🎤" } else { "➤" }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct MessageBubbleProps {
    message: database::Message,
}

#[component]
fn MessageBubble(props: MessageBubbleProps) -> Element {
    let is_self = props.message.sender_id == "self";
    
    rsx! {
        div {
            style: if is_self {
                "display: flex; justify-content: flex-end; margin-bottom: 8px;"
            } else {
                "display: flex; justify-content: flex-start; margin-bottom: 8px;"
            },
            div {
                style: "max-width: 70%;",
                div {
                    class: "glass-container animate-slide-in",
                    style: if is_self {
                        "padding: 12px 16px; border-radius: 18px 18px 4px 18px; background: var(--accent-color); color: white; border: 1px solid var(--accent-color);"
                    } else {
                        "padding: 12px 16px; border-radius: 18px 18px 18px 4px;"
                    },
                    
                    div { "{props.message.content}" }
                    
                    div {
                        style: "font-size: 11px; opacity: 0.7; margin-top: 4px; text-align: right;",
                        "{props.message.timestamp.split('T').next().unwrap_or(&props.message.timestamp)}"
                    }
                }
            }
        }
    }
}