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
pub fn ImageUploader29() -> Element {
    let image_data_url = use_signal(|| None::<String>);
    let rois = use_signal(|| Vec::<ROI>::new());
    let scale = use_signal(|| 1.0f32);
    let image_width = use_signal(|| 0f32);
    let image_height = use_signal(|| 0f32);

    let drag_start = use_signal(|| None::<(i32, i32)>);
    let drag_current = use_signal(|| None::<(i32, i32)>);

    // Mouse scroll zoom
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
        to_owned![drag_start, drag_current, rois, scale];
        move |evt: MouseEvent| {
            let coords = evt.data().element_coordinates();
            let scale_val = scale();
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
                let x = x0.min(x1);
                let y = y0.min(y1);
                let width = (x1 - x0).abs().max(1);
                let height = (y1 - y0).abs().max(1);

                rois.with_mut(|r| {
                    r.push(ROI {
                        x,
                        y,
                        width,
                        height,
                    });
                });

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(data_url) = image_data_url() {
                    if let Some(base64_data) = data_url.split(',').nth(1) {
                        if let Ok(image_bytes) = general_purpose::STANDARD.decode(base64_data) {
                            if let Ok(img) = image::load_from_memory(&image_bytes) {
                                let rgba_img = img.to_rgba8();
                                for dy in 0..height {
                                    for dx in 0..width {
                                        let px = x + dx;
                                        let py = y + dy;
                                        let _ = rgba_img.get_pixel_checked(px as u32, py as u32);
                                    }
                                }
                            }
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

                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let (w, h) = img.dimensions();
                        image_width.set(w as f32);
                        image_height.set(h as f32);
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
        let left = (x as f32 * scale_val).round();
        let top = (y as f32 * scale_val).round();
        let w = (width as f32 * scale_val).round();
        let h = (height as f32 * scale_val).round();
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
                "Upload Image"
            }

            if let Some(url) = image_data_url() {
                div { class: "mt-4",
                    div { class: "flex gap-2 mb-2",
                        span { "Use mouse scroll to zoom" }
                    }

                    div {
                        class: "relative border overflow-hidden mx-auto",
                        style: "width: 600px; height: 400px;",
                        onmousedown: on_mouse_down,
                        onmousemove: on_mouse_move,
                        onmouseup: on_mouse_up,
                        onwheel: on_wheel,

                        div {
                            style: "transform: scale({scale()}); transform-origin: top left; position: absolute;",
                            img {
                                src: "{url}",
                                class: "select-none pointer-events-none",
                                style: "width: {image_width()}px; height: {image_height()}px;",
                            }

                            // // Existing ROIs
                            // for roi in rois.read().iter() {
                            //     let left = roi.x as f32;
                            //     let top = roi.y as f32;
                            //     let width = roi.width as f32;
                            //     let height = roi.height as f32;

                            //     div {
                            //         class: "absolute border-2 border-red-500 pointer-events-none",
                            //         style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px;",
                            //     }
                            // }

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

                            // Dragging preview
                            {dragging_preview}
                        }
                    }
                }
            }
        }
    }
}
