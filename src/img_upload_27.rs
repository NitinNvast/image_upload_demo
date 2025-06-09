use crate::dioxus_elements::geometry::WheelDelta;
use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use image::GenericImageView;
use rfd::FileDialog;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
struct ROI {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn point_in_roi(x: i32, y: i32, roi: &ROI) -> bool {
    x >= roi.x && y >= roi.y && x < roi.x + roi.width && y < roi.y + roi.height
}

#[component]
pub fn ImageUploader27() -> Element {
    let image_data_url = use_signal(|| None::<String>);
    let rois = use_signal(|| Vec::<ROI>::new());
    let scale = use_signal(|| 1.0f32);
    let image_width = use_signal(|| 0f32);
    let image_height = use_signal(|| 0f32);

    // ROI size signals
    let mut roi_width = use_signal(|| 16i32);
    let mut roi_height = use_signal(|| 16i32);

    let on_wheel = {
        to_owned![scale];
        move |evt: WheelEvent| {
            let delta = evt.data().delta();
            scale.with_mut(|s| match delta {
                WheelDelta::Pixels(pixels) => {
                    if pixels.y < 0.0 {
                        *s = (*s * 1.1).min(5.0);
                    } else {
                        *s = (*s / 1.1).max(0.2);
                    }
                }
                _ => {}
            });
        }
    };

    let on_mouse_down = {
        to_owned![
            rois,
            scale,
            image_width,
            image_height,
            roi_width,
            roi_height
        ];
        move |evt: MouseEvent| {
            let coords = evt.data().element_coordinates();
            let scale_val = scale();
            let img_w = image_width();
            let img_h = image_height();

            let x_f = coords.x / scale_val as f64;
            let y_f = coords.y / scale_val as f64;

            if x_f < 0.0 || y_f < 0.0 || x_f >= img_w as f64 || y_f >= img_h as f64 {
                return;
            }

            let x = x_f as i32;
            let y = y_f as i32;

            let shift_pressed = evt.modifiers().shift();
            let roi_w = roi_width();
            let roi_h = roi_height();

            let centered_x = x - roi_w / 2;
            let centered_y = y - roi_h / 2;
            let max_x = (img_w as i32).saturating_sub(1);
            let max_y = (img_h as i32).saturating_sub(1);

            let clamped_x = centered_x.clamp(0, max_x - roi_w + 1);
            let clamped_y = centered_y.clamp(0, max_y - roi_h + 1);

            if shift_pressed {
                rois.with_mut(|r| {
                    if let Some(index) = r.iter().position(|roi| point_in_roi(x, y, roi)) {
                        r.remove(index);
                        println!("ROI deselected at ({}, {})", x, y);
                    }
                });
            } else {
                rois.with_mut(|r| {
                    r.push(ROI {
                        x: clamped_x,
                        y: clamped_y,
                        width: roi_w,
                        height: roi_h,
                    });
                    println!("ROI added at ({}, {})", clamped_x, clamped_y);
                });
            }
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let pick_image = {
        to_owned![image_data_url, rois, scale, image_width, image_height];
        move |_| {
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
                    let encoded = general_purpose::STANDARD.encode(bytes.clone());
                    let data_url = format!("data:{};base64,{}", mime, encoded);
                    image_data_url.set(Some(data_url));
                    rois.set(vec![]);
                    scale.set(1.0);

                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let (w, h) = img.dimensions();
                        image_width.set(w as f32);
                        image_height.set(h as f32);
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "p-4 font-sans space-y-4",
            div { class: "flex items-center space-x-4",
                button {
                    onclick: pick_image,
                    class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                    "Upload Image"
                }

                label {
                    class: "text-sm",
                    "ROI Width:",
                    input {
                        r#type: "number",
                        min: "16",
                        max: "512",
                        value: "{roi_width()}",
                        class: "ml-2 border rounded px-2 py-1 w-20",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<i32>() {
                                roi_width.set(val.clamp(16, 512));
                            }
                        }
                    }
                }

                label {
                    class: "text-sm",
                    "ROI Height:",
                    input {
                        r#type: "number",
                        min: "16",
                        max: "512",
                        value: "{roi_height()}",
                        class: "ml-2 border rounded px-2 py-1 w-20",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<i32>() {
                                roi_height.set(val.clamp(16, 512));
                            }
                        }
                    }
                }
            }

            if let Some(url) = image_data_url() {
                div {
                    style: "width: 640px; height: 440px; overflow: auto; border: 2px solid #ccc; margin: auto;",

                    div {
                        class: "relative",
                        style: "width: 600px; height: 400px; position: relative;",

                        div {
                            class: "relative border",
                            style: "width: {(image_width() * scale())}px; height: {(image_height() * scale())}px;",
                            onmousedown: on_mouse_down,
                            onwheel: on_wheel,

                            img {
                                src: "{url}",
                                class: "select-none pointer-events-none",
                                style: "width: {(image_width() * scale())}px; height: {(image_height() * scale())}px;",
                            }

                            {
                                rois.read().iter().map(|roi| {
                                    let scale_val = scale();
                                    let left = (roi.x as f32 * scale_val).round();
                                    let top = (roi.y as f32 * scale_val).round();
                                    let width = (roi.width as f32 * scale_val).round();
                                    let height = (roi.height as f32 * scale_val).round();
                                    rsx! {
                                        div {
                                            class: "absolute border-2 border-red-500 pointer-events-none",
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
}
