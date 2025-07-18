// Updated ImageUploader31 component with optional Subsample ROI placement
use crate::dioxus_elements::geometry::WheelDelta;
use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;

use opencv::{
    core::{Rect, Scalar, Vec3b, Vector},
    imgcodecs::{imdecode, imencode, IMREAD_COLOR},
    imgproc::rectangle,
    prelude::*,
};

fn point_in_rect(x: i32, y: i32, roi: &Rect) -> bool {
    x >= roi.x && y >= roi.y && x < roi.x + roi.width && y < roi.y + roi.height
}

#[component]
pub fn ImageUploader31() -> Element {
    let image_data_url = use_signal(|| None::<String>);
    let rois = use_signal(|| Vec::<Rect>::new());
    let mut scale = use_signal(|| 1.0f32);
    let image_width = use_signal(|| 0f32);
    let image_height = use_signal(|| 0f32);

    let drag_start = use_signal(|| None::<(i32, i32)>);
    let drag_current = use_signal(|| None::<(i32, i32)>);
    let mut subsample_mode = use_signal(|| true);
    let mut subsample_grayscale = use_signal(|| false);
    let mut subsample_rgb = use_signal(|| true);

    let all_image_paths = use_signal(|| Vec::<std::path::PathBuf>::new());
    let mut current_index = use_signal(|| 0usize);

    // ROI size signals
    let mut roi_width = use_signal(|| 16i32);
    let mut roi_height = use_signal(|| 16i32);

    let mut original_mat= use_signal(|| None::<Mat>);


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
        to_owned![scale, image_width, image_height, drag_start, drag_current, rois, subsample_mode, subsample_grayscale, subsample_rgb, roi_width, roi_height, original_mat];
        move |evt: MouseEvent| {
            let coords = evt.data().element_coordinates();
            let shift_pressed = evt.data().modifiers().shift();
            let scale_val = scale();
            let x = (coords.x / scale_val as f64) as i32;
            let y = (coords.y / scale_val as f64) as i32;

            if x >= 0 && y >= 0 && x < image_width() as i32 && y < image_height() as i32 {
                if subsample_mode() {
                    let w = roi_width();
                    let h = roi_height();
                    let roi = Rect::new(x - (w / 2), y - (h / 2), w, h); // ⬅️ Centered ROI

                    // rois.with_mut(|r| r.push(roi));
                    if shift_pressed {
                        rois.with_mut(|r| {
                            if let Some(index) = r.iter().position(|existing_roi| {
                                point_in_rect(x, y, existing_roi)
                            }) {
                                r.remove(index);
                            }
                        });
                    } else {
                        rois.with_mut(|r| r.push(roi));
                    }

                    if let Some(image) = &*original_mat.read() {
                        if subsample_grayscale() {
                            println!("🟢 Grayscale ROI @ ({x},{y}) ============>");
                            let mut count = 0;
                            for row in 0..h {
                                for col in 0..w {
                                    if count >= 256 {
                                        break;
                                    }
                                    match image.at_2d::<u8>(y + row, x + col) {
                                        Ok(val) => print!("{:3}, ", val),
                                        Err(_) => print!("Err, "),
                                    }
                                    count += 1;
                                }
                                if count >= 256 {
                                    break;
                                }
                                println!();
                            }
                        } else if subsample_rgb() {
                            println!("🎨  R Channel ({}x{})", w, h);
                            for row in 0..h {
                                for col in 0..w {
                                    match image.at_2d::<Vec3b>(y + row, x + col) {
                                        Ok(val) => print!("{:4}", val[0]), // R channel
                                        Err(_) => print!("Err "),
                                    }
                                }
                                println!();
                            }

                            println!("🎨  G Channel ({}x{})", w, h);
                            for row in 0..h {
                                for col in 0..w {
                                    match image.at_2d::<Vec3b>(y + row, x + col) {
                                        Ok(val) => print!("{:4}", val[1]), // G channel
                                        Err(_) => print!("Err "),
                                    }
                                }
                                println!();
                            }

                            println!("🎨  B Channel ({}x{})", w, h);
                            for row in 0..h {
                                for col in 0..w {
                                    match image.at_2d::<Vec3b>(y + row, x + col) {
                                        Ok(val) => print!("{:4}", val[2]), // B channel
                                        Err(_) => print!("Err "),
                                    }
                                }
                                println!();
                            }

                            println!("🔗  Combined RGB ({}x{})", w, h);
                            for row in 0..h {
                                for col in 0..w {
                                    match image.at_2d::<Vec3b>(y + row, x + col) {
                                        Ok(val) => print!("[{:3}, {:3}, {:3}] ", val[0], val[1], val[2]), // R, G, B
                                        Err(_) => print!("[Err] "),
                                    }
                                }
                                println!();
                            }
                        }
                    }
                } else {
                    drag_start.set(Some((x, y)));
                    drag_current.set(Some((x, y)));
                }
            }
        }
    };


    let on_mouse_move = {
        to_owned![scale, drag_current];
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
        to_owned![drag_start, drag_current, rois, image_width, image_height, subsample_mode, subsample_grayscale, subsample_rgb, original_mat];
        move |evt: MouseEvent| {
            if subsample_mode() {
                drag_start.set(None);
                drag_current.set(None);
                return;
            }

            if let (Some((x0, y0)), Some((x1, y1))) = (drag_start(), drag_current()) {
                let shift_pressed = evt.data().modifiers().shift();

                let x = x0.min(x1);
                let y = y0.min(y1);
                let roi_width = (x1 - x0).abs();
                let roi_height = (y1 - y0).abs();

                let max_x = (image_width() as i32).saturating_sub(1);
                let max_y = (image_height() as i32).saturating_sub(1);
                let clamped_x = x.clamp(0, max_x);
                let clamped_y = y.clamp(0, max_y);

                if shift_pressed {
                    rois.with_mut(|r| {
                        if let Some(index) = r
                            .iter()
                            .position(|roi| point_in_rect(clamped_x, clamped_y, roi))
                        {
                            r.remove(index);
                        }
                    });
                } else if roi_width > 0 && roi_height > 0 {
                    let clamped_x = x.clamp(0, max_x - roi_width + 1);
                    let clamped_y = y.clamp(0, max_y - roi_height + 1);

                    rois.with_mut(|r| {
                        r.push(Rect::new(clamped_x, clamped_y, roi_width, roi_height));
                    });

                    if let Some(mat) = original_mat.read().as_ref() {
                        let roi = Rect::new(clamped_x, clamped_y, roi_width, roi_height);

                        for y in roi.y..roi.y + roi.height {
                            for x in roi.x..roi.x + roi.width {
                                if y < mat.rows() && x < mat.cols() {
                                    match mat.typ() {
                                        // Grayscale image
                                        opencv::core::CV_8UC1 => {
                                            if let Ok(val) = mat.at_2d::<u8>(y, x) {
                                                println!("Pixel at ({}, {}): Gray={}", x, y, *val);
                                            }
                                        }
                                        // RGB image
                                        opencv::core::CV_8UC3 => {
                                            if let Ok(p) = mat.at_2d::<Vec3b>(y, x) {
                                                println!(
                                                    "Pixel at ({}, {}): R={} G={} B={}",
                                                    x, y, p[2], p[1], p[0]
                                                );
                                            }
                                        }
                                        _ => {
                                            println!("Unsupported image format at ({}, {})", x, y);
                                        }
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
        to_owned![image_data_url, rois, scale, image_width, image_height, all_image_paths, current_index];
        move |_| {
            spawn({
                to_owned![image_data_url, rois, scale, image_width, image_height, all_image_paths, current_index];
                async move {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        if let Some(folder) = path.parent() {
                            if let Ok(entries) = fs::read_dir(folder) {
                                let mut image_paths = vec![];

                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                    if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                                        image_paths.push(path);
                                    }
                                }

                                image_paths.sort();
                                let selected_index = image_paths.iter().position(|p| p == &path).unwrap_or(0);
                                current_index.set(selected_index);
                                all_image_paths.set(image_paths);

                                load_image(
                                    &all_image_paths()[selected_index],
                                    &mut image_data_url,
                                    &mut image_width,
                                    &mut image_height,
                                    &mut rois,
                                    &mut scale,
                                    true,
                                    false,
                                    true,
                                    &mut original_mat
                                ).await;
                            }
                        }
                    }
                }
            });
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    async fn load_image(
        path: &std::path::PathBuf,
        image_data_url: &mut Signal<Option<String>>,
        image_width: &mut Signal<f32>,
        image_height: &mut Signal<f32>,
        rois: &mut Signal<Vec<Rect>>,
        scale: &mut Signal<f32>,
        subsample_mode: bool,
        subsample_grayscale: bool,
        subsample_rgb: bool,
        original_mat: &mut Signal<Option<Mat>>,
    ) {
        if let Ok(bytes) = tokio::fs::read(path).await {
            let mime = match path.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("webp") => "image/webp",
                _ => "application/octet-stream",
            };

            if let Ok(mat) = Mat::from_slice(&bytes) {
                if let Ok(mut image) = imdecode(&mat, IMREAD_COLOR) {
                    let original_image= image.clone();
                    // Handle subsampling mode
                    if subsample_mode {
                        if subsample_grayscale {
                            use opencv::{core::AlgorithmHint, imgproc};

                            let mut gray = Mat::default();
                            if imgproc::cvt_color(&image, &mut gray, imgproc::COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT).is_ok() {
                                image = gray; // Replace original image with grayscale
                            }
                        } else if subsample_rgb {
                            use opencv::{core::AlgorithmHint, imgproc};

                            let mut rgb = Mat::default();
                            if imgproc::cvt_color(&image, &mut rgb, imgproc::COLOR_BGR2RGB, 0, AlgorithmHint::ALGO_HINT_DEFAULT).is_ok() {
                                image = rgb;
                            }
                        }
                    }

                    let (w, h) = (image.cols(), image.rows());
                    image_width.set(w as f32);
                    image_height.set(h as f32);
                    original_mat.set(Some(image.clone()));

                    let mut buf = Vector::new();
                    // Use ".jpg" regardless of format for simplicity
                    if imencode(".jpg", &original_image, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.to_vec());
                        let data_url = format!("data:{};base64,{}", mime, encoded);
                        image_data_url.set(Some(data_url));
                        rois.set(vec![]);
                        scale.set(1.0);
                    }
                }
            }
        }
    }


    let dragging_preview = if let (Some((x0, y0)), Some((x1, y1))) = (drag_start(), drag_current()) {
        let scale_val = scale();
        let x = x0.min(x1);
        let y = y0.min(y1);
        let width = (x1 - x0).abs();
        let height = (y1 - y0).abs();
        let roi = Rect::new(x, y, width, height);
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
            div { class: "flex gap-2 mb-4",
                button { onclick: pick_image, class: "px-4 py-2 bg-indigo-600 text-white rounded", "Upload Image" },
                button { onclick: move |_| subsample_mode.set(!subsample_mode()), class: "px-4 py-2 bg-yellow-500 text-white rounded", "Toggle Subsample" },
                button { onclick: move |_| scale.with_mut(|s| *s *= 1.1), class: "px-2 py-1 bg-green-600 text-white rounded", "+" },
                button { onclick: move |_| scale.with_mut(|s| *s /= 1.1), class: "px-2 py-1 bg-red-600 text-white rounded", "-" },
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
                            onmousemove: on_mouse_move,
                            onmouseup: on_mouse_up,
                            // onwheel: on_wheel,

                            img {
                                src: "{url}",
                                class: "select-none pointer-events-none",
                                style: "width: {(image_width() * scale())}px; height: {(image_height() * scale())}px;",
                            }

                            { rois.read().iter().map(|roi| {
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
                            }) }

                            {dragging_preview}
                        }
                    }
                }

                 // ✅ Show navigation only if image is loaded
                div { class: "flex gap-2 mt-4 justify-center",
                    button {
                        disabled: *current_index.read() == 0,
                        onclick: move |_| {
                            let idx = *current_index.read();
                            if idx > 0 {
                                let new_index = idx - 1;
                                current_index.set(new_index);
                                spawn({
                                    to_owned![all_image_paths, image_data_url, image_width, image_height, rois, scale];
                                    async move {
                                        load_image(
                                            &all_image_paths()[new_index],
                                            &mut image_data_url,
                                            &mut image_width,
                                            &mut image_height,
                                            &mut rois,
                                            &mut scale,
                                            true,
                                            false,
                                            true,
                                            &mut original_mat
                                        ).await;
                                    }
                                });
                            }
                        },
                        "⏮ Prev"
                    }

                    button {
                        disabled: *current_index.read() + 1 >= all_image_paths().len(),
                        onclick: move |_| {
                            let idx = *current_index.read();
                            if idx + 1 < all_image_paths().len() {
                                let new_index = idx + 1;
                                current_index.set(new_index);
                                spawn({
                                    to_owned![all_image_paths, image_data_url, image_width, image_height, rois, scale];
                                    async move {
                                        load_image(
                                            &all_image_paths()[new_index],
                                            &mut image_data_url,
                                            &mut image_width,
                                            &mut image_height,
                                            &mut rois,
                                            &mut scale,
                                            true,
                                            false,
                                            true,
                                            &mut original_mat
                                        ).await;
                                    }
                                });
                            }
                        },
                        "Next ⏭"
                    }
                }

            }
        }
    }
}
