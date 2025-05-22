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
pub fn ImageUploader22() -> Element {
    let mut image_data_url = use_signal(|| None::<String>);
    let mut rois = use_signal(|| Vec::<ROI>::new());
    let mut scale = use_signal(|| 1.0f32);

    let mut drag_start = use_signal(|| None::<(i32, i32)>);
    let mut drag_current = use_signal(|| None::<(i32, i32)>);

    let zoom_in = move |_| scale.with_mut(|s| *s *= 1.1);
    let zoom_out = move |_| scale.with_mut(|s| *s /= 1.1);

    let on_mouse_down = move |evt: MouseEvent| {
        let coords = evt.data().element_coordinates();
        let scale_val = scale();
        let x = (coords.x / scale_val as f64) as i32;
        let y = (coords.y / scale_val as f64) as i32;
        drag_start.set(Some((x, y)));
        drag_current.set(Some((x, y)));
    };

    let on_mouse_move = move |evt: MouseEvent| {
        if drag_start().is_some() {
            let coords = evt.data().element_coordinates();
            let scale_val = scale();
            let x = (coords.x / scale_val as f64) as i32;
            let y = (coords.y / scale_val as f64) as i32;
            drag_current.set(Some((x, y)));
        }
    };

    let on_mouse_up = move |_evt: MouseEvent| {
        if let (Some((x0, y0)), Some((x1, y1))) = (drag_start(), drag_current()) {
            let x = x0.min(x1);
            let y = y0.min(y1);
            let width = (x1 - x0).abs();
            let height = (y1 - y0).abs();

            let x = if x <= 0 { 20 } else { x };
            let y = if y <= 0 { 20 } else { y };

            rois.with_mut(|r| {
                r.push(ROI {
                    x,
                    y,
                    width,
                    height,
                })
            });

            println!("ROI: ({}, {}, {}, {})", x, y, width, height);

            // Extract RGBA values from image
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(data_url) = image_data_url() {
                if let Some(base64_data) = data_url.split(',').nth(1) {
                    if let Ok(image_bytes) = general_purpose::STANDARD.decode(base64_data) {
                        if let Ok(img) = image::load_from_memory(&image_bytes) {
                            let rgba_img = img.to_rgba8();
                            let mut rgba_values = vec![];
                            for dy in 0..height {
                                for dx in 0..width {
                                    let px = x + dx;
                                    let py = y + dy;
                                    if let Some(pixel) =
                                        rgba_img.get_pixel_checked(px as u32, py as u32)
                                    {
                                        rgba_values.push((px, py, pixel.0));
                                    }
                                }
                            }

                            println!("Extracted RGBA pixels in ROI:");
                            for (px, py, rgba) in rgba_values.iter() {
                                println!(
                                    "Pixel ({}, {}): R={}, G={}, B={}, A={}",
                                    px, py, rgba[0], rgba[1], rgba[2], rgba[3]
                                );
                            }
                        } else {
                            println!("Error: Failed to decode image.");
                        }
                    } else {
                        println!("Error: Failed to decode base64.");
                    }
                }
            }
        }
        drag_start.set(None);
        drag_current.set(None);
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
                scale.set(1.0);
            }
        }
    };

    let dragging_preview = if let (Some((x0, y0)), Some((x1, y1))) = (drag_start(), drag_current())
    {
        let scale_val = scale();
        let x = x0.min(x1);
        let y = y0.min(y1);
        let width = (x1 - x0).abs();
        let height = (y1 - y0).abs();
        let roi = ROI {
            x,
            y,
            width,
            height,
        };
        let left = (roi.x as f32 * scale_val).round();
        let top = (roi.y as f32 * scale_val).round();
        let w = (roi.width as f32 * scale_val).round();
        let h = (roi.height as f32 * scale_val).round();
        Some(rsx! {
            div {
                class: "absolute border-2 border-blue-400 bg-blue-200 bg-opacity-30 pointer-events-none",
                style: "left: {left}px; top: {top}px; width: {w}px; height: {h}px;",
            }
        })
    } else {
        None
    };

    rsx! {
        div { class: "p-4 font-sans",
            button {
                onclick: pick_image,
                class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                "ImageUploader22"
            }

            if let Some(url) = image_data_url() {
                div { class: "mt-4",
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

                    div {
                        class: "relative inline-block",
                        onmousedown: on_mouse_down,
                        onmousemove: on_mouse_move,
                        onmouseup: on_mouse_up,

                        // Image display
                        img {
                            src: "{url}",
                            class: "border transform origin-top-left select-none pointer-events-none",
                            style: "max-width: none; transform: scale({scale()});",
                        }

                        // Existing ROIs
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
                                            class: "absolute border-2 border-red-500 pointer-events-none",
                                            style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px;",
                                        }
                                    }
                                })
                        }

                        // Drag preview overlay
                        {dragging_preview}
                    }
                }
            }
        }
    }
}
