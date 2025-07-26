use dioxus::prelude::*;
use crate::{Route, database};

#[component]
pub fn Profile() -> Element {
    let mut username = use_signal(|| "User".to_string());
    let _avatar_path = use_signal(|| None::<String>);
    let mut device_name = use_signal(|| "My Device".to_string());
    let mut status_message = use_signal(|| "Available".to_string());

    // Load profile data from settings
    use_effect(move || {
        #[cfg(feature = "desktop")]
        {
            if let Ok(settings) = database::get_settings() {
                username.set(settings.username);
            }
        }
        // For web version, use defaults
    });

    rsx! {
        div {
            class: "glass-container animate-slide-in",
            style: "
                display: flex; 
                flex-direction: column; 
                height: 100vh; 
                border-radius: 0;
                border: none;
            ",
            
            // Header
            div {
                class: "glass-container",
                style: "
                    padding: 16px 20px; 
                    border-bottom: 1px solid var(--glass-border);
                    border-radius: 0;
                    border: none;
                    border-bottom: 1px solid var(--glass-border);
                ",
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    Link {
                        to: Route::PeerList {},
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
                        ",
                        "←"
                    }
                    h1 {
                        style: "
                            font-size: 24px; 
                            font-weight: 700; 
                            color: var(--text-primary);
                            margin: 0;
                        ",
                        "Profile"
                    }
                }
            }

            // Profile content
            div {
                style: "
                    flex: 1; 
                    padding: 20px; 
                    overflow-y: auto;
                ",
                div {
                    style: "max-width: 500px; margin: 0 auto;",
                    
                    // Avatar section
                    div {
                        class: "glass-container",
                        style: "
                            padding: 32px 20px; 
                            border-radius: var(--border-radius);
                            margin-bottom: 20px;
                            text-align: center;
                        ",
                        div {
                            style: "margin-bottom: 24px;",
                            div {
                                style: "
                                    width: 120px; 
                                    height: 120px; 
                                    border-radius: 50%; 
                                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    color: white;
                                    font-weight: 700;
                                    font-size: 48px;
                                    margin: 0 auto 16px auto;
                                ",
                                "{username.read().chars().next().unwrap_or('?').to_uppercase()}"
                            }
                            button {
                                class: "glass-button",
                                style: "
                                    padding: 8px 16px; 
                                    font-size: 14px;
                                ",
                                "📷 Change Avatar"
                            }
                        }
                        h2 {
                            style: "
                                font-size: 24px; 
                                font-weight: 600; 
                                color: var(--text-primary);
                                margin: 0 0 8px 0;
                            ",
                            "{username.read()}"
                        }
                        div {
                            style: "
                                font-size: 14px; 
                                color: var(--success-color);
                                margin-bottom: 8px;
                            ",
                            "🟢 Online"
                        }
                        div {
                            style: "
                                font-size: 14px; 
                                color: var(--text-secondary);
                            ",
                            "{status_message.read()}"
                        }
                    }

                    // Profile Details section
                    div {
                        class: "glass-container",
                        style: "
                            padding: 20px; 
                            border-radius: var(--border-radius);
                            margin-bottom: 20px;
                        ",
                        h3 {
                            style: "
                                margin: 0 0 16px 0; 
                                font-size: 18px; 
                                font-weight: 600; 
                                color: var(--text-primary);
                            ",
                            "Profile Details"
                        }
                        div {
                            style: "margin-bottom: 16px;",
                            label {
                                style: "
                                    display: block; 
                                    margin-bottom: 6px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                ",
                                "Display Name"
                            }
                            input {
                                class: "glass-input",
                                r#type: "text",
                                style: "width: 100%;",
                                value: "{username.read()}",
                                oninput: move |evt| username.set(evt.value()),
                            }
                        }
                        div {
                            style: "margin-bottom: 16px;",
                            label {
                                style: "
                                    display: block; 
                                    margin-bottom: 6px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                ",
                                "Device Name"
                            }
                            input {
                                class: "glass-input",
                                r#type: "text",
                                style: "width: 100%;",
                                value: "{device_name.read()}",
                                oninput: move |evt| device_name.set(evt.value()),
                            }
                        }
                        div {
                            style: "margin-bottom: 16px;",
                            label {
                                style: "
                                    display: block; 
                                    margin-bottom: 6px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                ",
                                "Status Message"
                            }
                            input {
                                class: "glass-input",
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: "What's on your mind?",
                                value: "{status_message.read()}",
                                oninput: move |evt| status_message.set(evt.value()),
                            }
                        }
                    }

                    // Network Info section
                    div {
                        class: "glass-container",
                        style: "
                            padding: 20px; 
                            border-radius: var(--border-radius);
                            margin-bottom: 20px;
                        ",
                        h3 {
                            style: "
                                margin: 0 0 16px 0; 
                                font-size: 18px; 
                                font-weight: 600; 
                                color: var(--text-primary);
                            ",
                            "Network Information"
                        }
                        div {
                            style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                            div {
                                div {
                                    style: "
                                        font-size: 12px; 
                                        color: var(--text-secondary); 
                                        margin-bottom: 4px;
                                        font-weight: 500;
                                    ",
                                    "IP Address"
                                }
                                div {
                                    style: "
                                        font-size: 14px; 
                                        color: var(--text-primary);
                                        font-family: monospace;
                                    ",
                                    "192.168.1.100"
                                }
                            }
                            div {
                                div {
                                    style: "
                                        font-size: 12px; 
                                        color: var(--text-secondary); 
                                        margin-bottom: 4px;
                                        font-weight: 500;
                                    ",
                                    "Port"
                                }
                                div {
                                    style: "
                                        font-size: 14px; 
                                        color: var(--text-primary);
                                        font-family: monospace;
                                    ",
                                    "8081"
                                }
                            }
                            div {
                                div {
                                    style: "
                                        font-size: 12px; 
                                        color: var(--text-secondary); 
                                        margin-bottom: 4px;
                                        font-weight: 500;
                                    ",
                                    "Network"
                                }
                                div {
                                    style: "
                                        font-size: 14px; 
                                        color: var(--text-primary);
                                    ",
                                    "Local Network"
                                }
                            }
                            div {
                                div {
                                    style: "
                                        font-size: 12px; 
                                        color: var(--text-secondary); 
                                        margin-bottom: 4px;
                                        font-weight: 500;
                                    ",
                                    "Status"
                                }
                                div {
                                    style: "
                                        font-size: 14px; 
                                        color: var(--success-color);
                                        font-weight: 500;
                                    ",
                                    "🟢 Connected"
                                }
                            }
                        }
                    }

                    // Privacy section
                    div {
                        class: "glass-container",
                        style: "
                            padding: 20px; 
                            border-radius: var(--border-radius);
                            margin-bottom: 20px;
                        ",
                        h3 {
                            style: "
                                margin: 0 0 16px 0; 
                                font-size: 18px; 
                                font-weight: 600; 
                                color: var(--text-primary);
                            ",
                            "Privacy & Security"
                        }
                        div {
                            style: "margin-bottom: 12px;",
                            label {
                                style: "
                                    display: flex; 
                                    align-items: center; 
                                    gap: 8px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                    cursor: pointer;
                                ",
                                input {
                                    r#type: "checkbox",
                                    checked: true,
                                }
                                "Enable End-to-End Encryption"
                            }
                        }
                        div {
                            style: "margin-bottom: 12px;",
                            label {
                                style: "
                                    display: flex; 
                                    align-items: center; 
                                    gap: 8px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                    cursor: pointer;
                                ",
                                input {
                                    r#type: "checkbox",
                                    checked: true,
                                }
                                "Auto-discover on Local Network"
                            }
                        }
                        div {
                            label {
                                style: "
                                    display: flex; 
                                    align-items: center; 
                                    gap: 8px; 
                                    font-weight: 500; 
                                    color: var(--text-primary);
                                    cursor: pointer;
                                ",
                                input {
                                    r#type: "checkbox",
                                    checked: false,
                                }
                                "Share Typing Status"
                            }
                        }
                    }

                    // Save button
                    div {
                        style: "text-align: center; margin-top: 32px;",
                        button {
                            class: "glass-button primary",
                            style: "
                                padding: 16px 32px; 
                                font-size: 16px; 
                                font-weight: 600;
                            ",
                            "Save Profile"
                        }
                    }
                }
            }
        }
    }
}
