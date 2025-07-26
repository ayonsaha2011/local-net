use dioxus::prelude::*;
use crate::{Route, database};

#[component]
pub fn Settings() -> Element {
    let mut settings = use_signal(|| database::Settings {
        username: "User".to_string(),
        app_mode: "system".to_string(),
        notifications: true,
        download_path: "./downloads".to_string(),
        auto_accept_files: false,
        max_file_size: None,
    });

    // Load settings from database or use defaults
    use_effect(move || {
        #[cfg(feature = "desktop")]
        {
            if let Ok(db_settings) = database::get_settings() {
                settings.set(db_settings);
            }
        }
        // For web version, we already have defaults set above
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
                        "Settings"
                    }
                }
            }

            // Settings content
            div {
                style: "
                    flex: 1; 
                    padding: 20px; 
                    overflow-y: auto;
                ",
                div {
                    style: "max-width: 600px; margin: 0 auto;",
                    
                    // Profile section
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
                            "Profile"
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
                                "Username"
                            }
                            input {
                                class: "glass-input",
                                r#type: "text",
                                style: "width: 100%;",
                                value: "{settings.read().username}",
                                oninput: move |evt| {
                                    let mut current = settings.read().clone();
                                    current.username = evt.value();
                                    settings.set(current);
                                },
                            }
                        }
                    }

                    // Appearance section
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
                            "Appearance"
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
                                "Theme"
                            }
                            select {
                                class: "glass-input",
                                style: "width: 100%;",
                                value: "{settings.read().app_mode}",
                                onchange: move |evt| {
                                    let mut current = settings.read().clone();
                                    current.app_mode = evt.value();
                                    settings.set(current);
                                },
                                option { value: "system", "System (Auto)" }
                                option { value: "light", "Light" }
                                option { value: "dark", "Dark" }
                            }
                        }
                    }

                    // File Transfer section
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
                            "File Transfer"
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
                                "Download Path"
                            }
                            div {
                                style: "display: flex; gap: 8px;",
                                input {
                                    class: "glass-input",
                                    r#type: "text",
                                    style: "flex: 1;",
                                    value: "{settings.read().download_path}",
                                    oninput: move |evt| {
                                        let mut current = settings.read().clone();
                                        current.download_path = evt.value();
                                        settings.set(current);
                                    },
                                }
                                button {
                                    class: "glass-button",
                                    style: "padding: 12px 16px;",
                                    "Browse"
                                }
                            }
                        }
                        div {
                            style: "margin-bottom: 16px;",
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
                                    checked: settings.read().auto_accept_files,
                                    onchange: move |evt| {
                                        let mut current = settings.read().clone();
                                        current.auto_accept_files = evt.checked();
                                        settings.set(current);
                                    },
                                }
                                "Auto-accept Files"
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
                                "Max File Size (MB)"
                            }
                            input {
                                class: "glass-input",
                                r#type: "number",
                                style: "width: 100%;",
                                value: "{settings.read().max_file_size.map(|s| s.to_string()).unwrap_or_else(|| String::new())}",
                                placeholder: "No limit",
                                oninput: move |evt| {
                                    let mut current = settings.read().clone();
                                    current.max_file_size = if evt.value().is_empty() {
                                        None
                                    } else {
                                        evt.value().parse().ok()
                                    };
                                    settings.set(current);
                                },
                            }
                        }
                    }

                    // Notifications section
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
                            "Notifications"
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
                                    checked: settings.read().notifications,
                                    onchange: move |evt| {
                                        let mut current = settings.read().clone();
                                        current.notifications = evt.checked();
                                        settings.set(current);
                                    },
                                }
                                "Enable Notifications"
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
                            "Save Settings"
                        }
                    }
                }
            }
        }
    }
}
