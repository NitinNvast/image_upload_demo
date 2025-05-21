use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;

#[component]
pub fn ImageUploader() -> Element {
    // Make signal mutable
    let mut image_data_url = use_signal(|| None::<String>);

    #[cfg(not(target_arch = "wasm32"))]
    let pick_image = move |_| {
        if let Some(path) = FileDialog::new().add_filter("Image", &["png"]).pick_file() {
            if let Ok(bytes) = fs::read(&path) {
                let mime = match path.extension().and_then(|e| e.to_str()) {
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("webp") => "image/webp",
                    _ => "application/octet-stream",
                };
                let encoded = general_purpose::STANDARD.encode(bytes);
                let data_url = format!("data:{};base64,{}", mime, encoded);
                // Update the signal value
                image_data_url.set(Some(data_url));
            }
        }
    };

    rsx! {
        div { class: "p-4 font-sans",
            button {
                onclick: pick_image,
                class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                "ImageUploader"
            }
            {
                if let Some(url) = image_data_url() {
                    Some(rsx! {
                        div { class: "mt-4",
                            img { src: "{url}", class: "max-w-[600px]" }
                        }
                    })
                } else {
                    None
                }
            }
        }
    }
}
