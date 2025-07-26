#![allow(non_snake_case)]
use dioxus::prelude::*;

mod components;
mod database;
#[cfg(feature = "desktop")]
mod network;
mod network_interface;
use components::peer_list::PeerList;
use components::chat_view::ChatView;
use components::settings::Settings;
use components::profile::Profile;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    PeerList {},
    #[route("/chat/:peer_id")]
    ChatView { peer_id: String },
    #[route("/settings")]
    Settings {},
    #[route("/profile")]
    Profile {},
}

fn main() {
    // Initialize the database
    #[cfg(feature = "desktop")]
    if let Err(err) = database::init_db() {
        eprintln!("Error initializing database: {}", err);
    }

    // Start the network in a background thread for desktop
    #[cfg(feature = "desktop")]
    {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                network::start_network().await;
            });
        });
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    println!("🎨 UI: App component rendering...");
    rsx! {
        head {
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no"
            }
        }
        style {
            "
            :root {{
                --glass-bg: rgba(255, 255, 255, 0.2);
                --glass-border: rgba(255, 255, 255, 0.3);
                --text-primary: #1a1a1a;
                --text-secondary: #666666;
                --accent-color: #007AFF;
                --error-color: #FF3B30;
                --success-color: #34C759;
                --shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
                --border-radius: 12px;
            }}

            @media (prefers-color-scheme: dark) {{
                :root {{
                    --glass-bg: rgba(0, 0, 0, 0.2);
                    --glass-border: rgba(255, 255, 255, 0.1);
                    --text-primary: #ffffff;
                    --text-secondary: #a0a0a0;
                }}
            }}

            * {{
                box-sizing: border-box;
                margin: 0;
                padding: 0;
            }}

            body {{
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
                color: var(--text-primary);
                margin: 0;
                overflow: hidden;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
            }}

            .glass-container {{
                background: var(--glass-bg);
                backdrop-filter: blur(20px);
                -webkit-backdrop-filter: blur(20px);
                border: 1px solid var(--glass-border);
                box-shadow: var(--shadow);
            }}

            .glass-input {{
                background: var(--glass-bg);
                backdrop-filter: blur(10px);
                -webkit-backdrop-filter: blur(10px);
                border: 1px solid var(--glass-border);
                border-radius: 8px;
                padding: 12px;
                color: var(--text-primary);
                outline: none;
                transition: all 0.2s ease;
            }}

            .glass-input:focus {{
                border-color: var(--accent-color);
                box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
            }}

            .glass-button {{
                background: var(--glass-bg);
                backdrop-filter: blur(10px);
                -webkit-backdrop-filter: blur(10px);
                border: 1px solid var(--glass-border);
                border-radius: 8px;
                padding: 12px 24px;
                color: var(--text-primary);
                cursor: pointer;
                transition: all 0.2s ease;
                font-weight: 500;
            }}

            .glass-button:hover {{
                background: rgba(255, 255, 255, 0.3);
                transform: translateY(-1px);
            }}

            .glass-button:active {{
                transform: translateY(0);
            }}

            .glass-button.primary {{
                background: var(--accent-color);
                color: white;
                border-color: var(--accent-color);
            }}

            .glass-button.primary:hover {{
                background: rgba(0, 122, 255, 0.8);
            }}

            .animate-slide-in {{
                animation: slideIn 0.3s ease-out;
            }}

            @keyframes slideIn {{
                from {{
                    opacity: 0;
                    transform: translateY(20px);
                }}
                to {{
                    opacity: 1;
                    transform: translateY(0);
                }}
            }}

            /* Mobile Responsive Styles */
            @media (max-width: 768px) {{
                body {{
                    margin: 0;
                    padding: 0;
                }}
                
                .glass-container {{
                    margin: 0;
                    border-radius: 0;
                    border-left: none;
                    border-right: none;
                }}
                
                .glass-button {{
                    padding: 14px 16px;
                    font-size: 16px;
                    min-height: 44px;
                    touch-action: manipulation;
                }}
                
                .glass-input {{
                    padding: 14px 16px;
                    font-size: 16px;
                    min-height: 44px;
                }}
                
                /* Larger touch targets for mobile */
                button, input, select, textarea {{
                    min-height: 44px;
                    font-size: 16px;
                }}
            }}
            
            @media (max-width: 480px) {{
                .glass-button {{
                    padding: 12px 14px;
                    font-size: 14px;
                }}
                
                .glass-input {{
                    padding: 12px 14px;
                    font-size: 14px;
                }}
            }}

            .typing-indicator {{
                display: flex;
                align-items: center;
                gap: 4px;
                padding: 8px 12px;
                margin: 4px 0;
            }}

            .typing-dot {{
                width: 6px;
                height: 6px;
                border-radius: 50%;
                background: var(--text-secondary);
                animation: typingAnimation 1.4s infinite;
            }}

            .typing-dot:nth-child(2) {{
                animation-delay: 0.2s;
            }}

            .typing-dot:nth-child(3) {{
                animation-delay: 0.4s;
            }}

            @keyframes typingAnimation {{
                0%, 60%, 100% {{
                    transform: translateY(0);
                    opacity: 0.4;
                }}
                30% {{
                    transform: translateY(-10px);
                    opacity: 1;
                }}
            }}

            .app-container {{
                height: 100vh;
                overflow: hidden;
            }}
            "
        }
        div {
            class: "app-container",
            Router::<Route> {}
        }
    }
}
