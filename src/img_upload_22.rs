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
pub fn ImageUploader22() -> Element {
    let image_data_url = use_signal(|| None::<String>);
    let rois = use_signal(|| Vec::<ROI>::new());
    let mut scale = use_signal(|| 1.0f32);

    // New signals for original image dimensions
    let image_width = use_signal(|| 0f32);
    let image_height = use_signal(|| 0f32);

    let drag_start = use_signal(|| None::<(i32, i32)>);
    let drag_current = use_signal(|| None::<(i32, i32)>);

    // Zoom with limits
    let zoom_in = move |_| scale.with_mut(|s| *s = (*s * 1.1).min(5.0));
    let zoom_out = move |_| scale.with_mut(|s| *s = (*s / 1.1).max(0.2));

    let on_mouse_down = {
        to_owned![drag_start, drag_current, rois, scale];
        move |evt: MouseEvent| {
            let coords = evt.data().element_coordinates();
            let scale_val = scale();
            // Mouse coords relative to image pixels (taking scale into account)
            let x = (coords.x / scale_val as f64) as i32;
            let y = (coords.y / scale_val as f64) as i32;

            let shift_pressed = evt.modifiers().shift();

            if shift_pressed {
                rois.with_mut(|r| {
                    if let Some(index) = r.iter().position(|roi| point_in_roi(x, y, roi)) {
                        r.remove(index);
                        println!("ROI deselected at ({}, {})", x, y);
                    }
                });
                return;
            }

            drag_start.set(Some((x, y)));
            drag_current.set(Some((x, y)));
        }
    };

    let on_mouse_move = {
        to_owned![drag_start, drag_current, scale];
        move |evt: MouseEvent| {
            if drag_start().is_some() {
                let coords = evt.data().element_coordinates();
                let scale_val = scale();
                let x = (coords.x / scale_val as f64) as i32;
                let y = (coords.y / scale_val as f64) as i32;
                drag_current.set(Some((x, y)));
            }
        }
    };

    let on_mouse_up = {
        to_owned![drag_start, drag_current, rois, image_data_url];
        move |_evt: MouseEvent| {
            if let (Some((x0, y0)), Some((x1, y1))) = (drag_start(), drag_current()) {
                let x = if (x1 - x0).abs() <= 0 || (y1 - y0).abs() <= 0 {
                    x0.min(x1) - 10
                } else {
                    x0.min(x1)
                };
                let y = if (y1 - y0).abs() <= 0 || (y1 - y0).abs() <= 0 {
                    y0.min(y1) - 10
                } else {
                    y0.min(y1)
                };
                let width = if (x1 - x0).abs() <= 0 {
                    20
                } else {
                    (x1 - x0).abs()
                };
                let height = if (y1 - y0).abs() <= 0 {
                    20
                } else {
                    (y1 - y0).abs()
                };

                rois.with_mut(|r| {
                    r.push(ROI {
                        x,
                        y,
                        width,
                        height,
                    })
                });

                println!("ROI: ({}, {}, {}, {})", x, y, width, height);

                // Extract RGBA values from image (optional debug)
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
                                    // Uncomment below to print pixel info
                                    // println!(
                                    //     "Pixel ({}, {}): R={}, G={}, B={}, A={}",
                                    //     px, py, rgba[0], rgba[1], rgba[2], rgba[3]
                                    // );
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

                    // Extract original dimensions
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let (w, h) = img.dimensions();
                        image_width.set(w as f32);
                        image_height.set(h as f32);
                    } else {
                        image_width.set(0.0);
                        image_height.set(0.0);
                    }
                }
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

    let w = (image_width() * scale()) as u32;
    let h = (image_height() * scale()) as u32;

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
                            class: "border select-none pointer-events-none",
                            width: "{w}",
                            height: "{h}",
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
