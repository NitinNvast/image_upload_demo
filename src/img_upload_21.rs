use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
struct ROI {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[component]
pub fn ImageUploader21() -> Element {
    let mut image_data_url = use_signal(|| None::<String>);
    let mut rois = use_signal(|| Vec::<ROI>::new());
    let mut scale = use_signal(|| 1.0f32);

    // Zoom controls
    let zoom_in = move |_| {
        scale.with_mut(|s| *s *= 1.1);
    };

    let zoom_out = move |_| {
        scale.with_mut(|s| *s /= 1.1);
    };

    // Handle image click to create ROI (adjust for scale)
    let on_image_click = move |evt: MouseEvent| {
        // Get mouse coords relative to the image element
        let point = evt.data().element_coordinates();

        let scale_val = scale();

        // Convert the clicked position from scaled image coords back to original image coords
        let x = (point.x / scale_val as f64) as i32;
        let y = (point.y / scale_val as f64) as i32;

        println!("Image Clicked at (scaled coords): x={}, y={}", x, y);

        rois.with_mut(|r| {
            // Add ROI centered around clicked point with fixed size (20x20)
            r.push(ROI {
                x: x - 10,
                y: y - 10,
                width: 20,
                height: 20,
            });
        });
    };

    #[cfg(not(target_arch = "wasm32"))]
    let pick_image = move |_| {
        if let Some(path) = FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            if let Ok(bytes) = fs::read(&path) {
                let mime = match path.extension().and_then(|e| e.to_str()) {
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("webp") => "image/webp",
                    _ => "application/octet-stream",
                };
                let encoded = general_purpose::STANDARD.encode(bytes);
                let data_url = format!("data:{};base64,{}", mime, encoded);
                image_data_url.set(Some(data_url));
                rois.set(vec![]);
                scale.set(1.0); // Reset zoom when new image is selected
            }
        }
    };

    // println!("🚀 ~ pubfnImageUploader21 ~ rois:{:#?}", rois);

    rsx! {
        div { class: "p-4 font-sans",

            button {
                onclick: pick_image,
                class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                "ImageUploader21"
            }

            if let Some(url) = image_data_url() {
                div { class: "mt-4",

                    // Zoom control buttons
                    div { class: "flex gap-2 mb-2",
                        button {
                            onclick: zoom_in,
                            class: "px-2 py-1 bg-green-600 text-white rounded",
                            "+"
                        }
                        button {
                            onclick: zoom_out,
                            class: "px-2 py-1 bg-red-600 text-white rounded",
                            "-"
                        }
                    }

                    div { class: "relative inline-block",

                        img {
                            src: "{url}",
                            class: "border transform origin-top-left",
                            style: "max-width: none; transform: scale({scale()});",
                            onclick: on_image_click,
                        }

                        {
                            rois.read()
                                .iter()
                                .map(|roi| {
                                    let scale_val = scale();
                                    let left = (roi.x as f32 * scale_val).round();
                                    let top = (roi.y as f32 * scale_val).round();
                                    let width = (roi.width as f32 * scale_val).round();
                                    let height = (roi.height as f32 * scale_val).round();
                                    rsx! {
                                        div {
                                            class: "absolute border-2 border-red-500",
                                            style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px;",
                                        }
                                    }
                                })
                        }
                    }
                }
            }
        }
    }
}
