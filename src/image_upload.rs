use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;

use opencv::{
    core::{
        bitwise_not, no_array, rotate, AlgorithmHint, Point, Size, Vector, BORDER_DEFAULT,
        ROTATE_90_CLOCKWISE,
    },
    imgcodecs::{imdecode, imencode, IMREAD_COLOR},
    imgproc,
    prelude::*,
};
fn decode_mat(bytes: &[u8]) -> opencv::Result<Mat> {
    let vec = Vector::from_slice(bytes);
    imdecode(&vec, IMREAD_COLOR)
}

#[component]
pub fn ImageUploader() -> Element {
    let mut image_data_url = use_signal(|| None::<String>);
    let mut original_image_bytes = use_signal(|| None::<Vec<u8>>);

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
                let encoded = general_purpose::STANDARD.encode(&bytes);
                let data_url = format!("data:{};base64,{}", mime, encoded);
                image_data_url.set(Some(data_url));
                original_image_bytes.set(Some(bytes));
            }
        }
    };

    let apply_blur = move |_event: dioxus::events::MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = imdecode(&Vector::from_slice(&bytes), IMREAD_COLOR) {
                let mut blurred = Mat::default();
                if imgproc::blur(
                    &input_mat,
                    &mut blurred,
                    Size::new(15, 15),
                    Point::new(-1, -1),
                    BORDER_DEFAULT,
                )
                .is_ok()
                {
                    let mut buf = Vector::new();
                    if imencode(".png", &blurred, &mut buf, &Vector::<i32>::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);
                        image_data_url.set(Some(data_url));
                    }
                }
            }
        }
    };

    let apply_resize = move |_event: dioxus::events::MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            let vec = Vector::from_slice(&bytes);

            if let Ok(input_mat) = imdecode(&vec, IMREAD_COLOR) {
                let mut resized = Mat::default();
                let new_size = Size::new(200, 200); // Resize to 200x200

                if imgproc::resize(
                    &input_mat,
                    &mut resized,
                    new_size,
                    0.0,
                    0.0,
                    imgproc::INTER_LINEAR,
                )
                .is_ok()
                {
                    let mut buf = Vector::new(); // this is Vector<u8>
                    if imencode(".png", &resized, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);
                        image_data_url.set(Some(data_url));
                    }
                }
            }
        }
    };

    let apply_grayscale = move |_event: MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = decode_mat(&bytes) {
                let mut gray = Mat::default();
                if imgproc::cvt_color(
                    &input_mat,
                    &mut gray,
                    imgproc::COLOR_BGR2GRAY,
                    0,
                    AlgorithmHint::ALGO_HINT_DEFAULT,
                )
                .is_ok()
                {
                    let mut buf = Vector::new();
                    if imencode(".png", &gray, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);
                        image_data_url.set(Some(data_url));
                    }
                }
            }
        }
    };

    let apply_invert = move |_event: MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = decode_mat(&bytes) {
                let mut inverted = Mat::default();
                if bitwise_not(&input_mat, &mut inverted, &no_array()).is_ok() {
                    let mut buf = Vector::new();
                    if imencode(".png", &inverted, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);
                        image_data_url.set(Some(data_url));
                    }
                }
            }
        }
    };

    let apply_edge_detect = move |_event: MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = decode_mat(&bytes) {
                let mut gray = Mat::default();
                if imgproc::cvt_color(
                    &input_mat,
                    &mut gray,
                    imgproc::COLOR_BGR2GRAY,
                    0,
                    AlgorithmHint::ALGO_HINT_DEFAULT,
                )
                .is_ok()
                {
                    let mut edges = Mat::default();
                    if imgproc::canny(&gray, &mut edges, 100.0, 200.0, 3, false).is_ok() {
                        let mut buf = Vector::new();
                        if imencode(".png", &edges, &mut buf, &Vector::new()).is_ok() {
                            let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                            let data_url = format!("data:image/png;base64,{}", encoded);
                            image_data_url.set(Some(data_url));
                        }
                    }
                }
            }
        }
    };

    let apply_rotate_90 = move |_event: MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = decode_mat(&bytes) {
                let mut rotated = Mat::default();

                // Rotate the image 90 degrees clockwise
                if rotate(&input_mat, &mut rotated, ROTATE_90_CLOCKWISE).is_ok() {
                    let mut buf = Vector::new();
                    if imencode(".png", &rotated, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);

                        // Update the image signal
                        image_data_url.set(Some(data_url));

                        // Optional: update the original bytes to allow chained effects
                        original_image_bytes.set(Some(buf.to_vec()));
                    }
                }
            }
        }
    };

    let apply_crop = move |_event: MouseEvent| {
        if let Some(bytes) = original_image_bytes().clone() {
            if let Ok(input_mat) = decode_mat(&bytes) {
                let roi = opencv::core::Rect::new(50, 50, 100, 100); // x, y, width, height
                if let Ok(cropped) = Mat::roi(&input_mat, roi) {
                    let mut buf = Vector::new();
                    if imencode(".png", &cropped, &mut buf, &Vector::new()).is_ok() {
                        let encoded = general_purpose::STANDARD.encode(buf.as_slice());
                        let data_url = format!("data:image/png;base64,{}", encoded);
                        image_data_url.set(Some(data_url));
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "p-4 font-sans",
            button {
                onclick: pick_image,
                class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                "Upload Image"
            }

            {
                if let Some(url) = image_data_url() {
                    Some(rsx! {
                        div { class: "mt-4",
                        img { src: "{url}", class: "max-w-[600px] border rounded shadow mb-4" }

                        div { class: "flex gap-4",
                            button {
                                onclick: apply_blur,
                                class: "px-4 py-2 bg-blue-500 text-white rounded",
                                "Apply Blur"
                            }
                            button {
                                onclick: apply_resize,
                                class: "px-4 py-2 bg-green-500 text-white rounded",
                                "Resize 200x200"
                            }
                            button {
                                onclick: apply_grayscale,
                                class: "px-4 py-2 bg-green-500 text-white rounded",
                                "Grayscale"
                            }
                            button {
                                onclick: apply_invert,
                                class: "mt-2 px-4 py-2 bg-yellow-600 text-white rounded",
                                "Invert Colors"
                            }
                            button {
                                onclick: apply_edge_detect,
                                class: "mt-2 px-4 py-2 bg-red-600 text-white rounded ml-2",
                                "Edge Detect"
                            }
                            button {
                                onclick: apply_crop,
                                class: "px-4 py-2 bg-blue-500 text-white rounded",
                                "Crop (ROI)"
                            }

                            button {
                                onclick: apply_rotate_90,
                                class: "px-4 py-2 bg-green-500 text-white rounded",
                                "Rotate 90°"
                            }
                        }
                    }

                    })
                } else {
                    None
                }
            }
        }
    }
}
