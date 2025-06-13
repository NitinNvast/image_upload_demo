use base64::engine::general_purpose;
use base64::Engine;
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use {
    opencv::{
        core::{Point, Rect, Scalar},
        highgui, imgcodecs, imgproc,
        prelude::*,
    },
    std::{
        sync::{mpsc, Arc, Mutex},
        thread,
    },
};

#[component]
pub fn RFD_Image_Upload() -> Element {
    let mut image_data_url = use_signal(|| None::<String>);

    #[cfg(not(target_arch = "wasm32"))]
    let pick_image = move |_| {
        // We clone a sender and send result back from thread
        let (tx, rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            if let Some(path) = FileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg"])
                .pick_file()
            {
                // Load image
                let original_mat =
                    imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_COLOR)
                        .expect("Failed to load image");
                let mut display_mat = original_mat.clone();

                let window = "Draw ROI";
                highgui::named_window(window, highgui::WINDOW_AUTOSIZE).unwrap();

                let roi_state = Arc::new(Mutex::new(None::<Rect>));
                let mat_arc = Arc::new(Mutex::new(display_mat.clone()));

                {
                    let roi_state = roi_state.clone();
                    let mat_arc = mat_arc.clone();

                    let mut drawing = false;
                    let mut start_point = Point::default();

                    highgui::set_mouse_callback(
                        window,
                        Some(Box::new(move |event, x, y, _flags| match event {
                            highgui::EVENT_LBUTTONDOWN => {
                                drawing = true;
                                start_point = Point::new(x, y);
                            }
                            highgui::EVENT_MOUSEMOVE => {
                                if drawing {
                                    let mat = {
                                        let mat_locked = mat_arc.lock().unwrap();
                                        mat_locked.clone()
                                    };
                                    let mut temp = mat.clone();
                                    let rect = Rect::new(
                                        start_point.x.min(x),
                                        start_point.y.min(y),
                                        (x - start_point.x).abs(),
                                        (y - start_point.y).abs(),
                                    );
                                    imgproc::rectangle(
                                        &mut temp,
                                        rect,
                                        Scalar::new(0.0, 255.0, 0.0, 0.0),
                                        1,
                                        imgproc::LINE_8,
                                        0,
                                    )
                                    .unwrap();
                                    highgui::imshow(window, &temp).unwrap();
                                }
                            }
                            highgui::EVENT_LBUTTONUP => {
                                drawing = false;
                                let roi = Rect::new(
                                    start_point.x.min(x),
                                    start_point.y.min(y),
                                    (x - start_point.x).abs(),
                                    (y - start_point.y).abs(),
                                );
                                *roi_state.lock().unwrap() = Some(roi);

                                {
                                    let mut mat = mat_arc.lock().unwrap();
                                    imgproc::rectangle(
                                        &mut *mat,
                                        roi,
                                        Scalar::new(0.0, 0.0, 255.0, 0.0),
                                        1,
                                        imgproc::LINE_8,
                                        0,
                                    )
                                    .unwrap();
                                    highgui::imshow(window, &*mat).unwrap();
                                }

                                println!("ROI selected: {:?}", roi);
                            }
                            _ => {}
                        })),
                    )
                    .unwrap();
                }

                highgui::imshow(window, &display_mat).unwrap();
                while highgui::wait_key(1).unwrap() != 27 {}
                highgui::destroy_window(window).unwrap();

                // Base64 conversion
                if let Ok(bytes) = fs::read(&path) {
                    let mime = match path.extension().and_then(|e| e.to_str()) {
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        _ => "application/octet-stream",
                    };
                    let encoded = general_purpose::STANDARD.encode(bytes);
                    let data_url = format!("data:{};base64,{}", mime, encoded);
                    let _ = tx.send(data_url);
                }
            }
        });

        // Spawn async in Dioxus to receive and update
        spawn(async move {
            if let Ok(data_url) = rx.recv() {
                image_data_url.set(Some(data_url));
            }
        });
    };

    rsx! {
        div { class: "p-4 font-sans",
            button {
                onclick: pick_image,
                class: "px-4 py-2 bg-indigo-600 text-white rounded text-2xl",
                "RFD Image Upload"
            }
            {
                if let Some(url) = image_data_url() {
                    Some(rsx! {
                        div { class: "mt-4",
                            img { src: "{url}", class: "max-w-[600px] border border-gray-400" }
                        }
                    })
                } else {
                    None
                }
            }
        }
    }
}
