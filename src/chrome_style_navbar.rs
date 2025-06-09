use dioxus::prelude::*;

#[component]
pub fn ChromeStyleNavbar() -> Element {
    let mut tabs = use_signal(|| {
        vec![Tab {
            id: 1,
            title: "Home".to_string(),
            active: true,
        }]
    });

    let next_id = use_signal(|| 2);

    let add_tab = {
        to_owned![tabs, next_id];
        move |_| {
            tabs.write().iter_mut().for_each(|t| t.active = false);
            tabs.write().push(Tab {
                id: next_id(),
                title: format!("Tab {}", next_id()),
                active: true,
            });
            next_id.set(next_id() + 1); // <--- fix here
        }
    };

    let mut activate_tab = {
        to_owned![tabs];
        move |id| {
            tabs.write().iter_mut().for_each(|t| t.active = t.id == id);
        }
    };

    let mut close_tab = {
        to_owned![tabs];
        move |id| {
            let mut new_list: Vec<Tab> = tabs().into_iter().filter(|t| t.id != id).collect();

            if !new_list.iter().any(|t| t.active) {
                if let Some(first) = new_list.first_mut() {
                    first.active = true;
                }
            }

            tabs.set(new_list);
        }
    };

    let rendered_tabs = tabs()
    .clone()
    .into_iter()
    .map(|tab| {
        let (bg_class, border_class, text_class, z_class) = if tab.active {
            (
                "bg-[#e0e0e0] hover:bg-[#d0d0d0]",
                "border-x border-t border-gray-300",
                "text-black",
                "z-10",
            )
        } else {
            (
                // "bg-[#e0e0e0] hover:bg-[#d0d0d0]",
                "bg-white",
                "border border-transparent",
                "text-gray-600",
                "z-0",
            )
        };

        let class_name = format!(
            "relative px-4 py-1 flex items-center gap-2 text-sm rounded-t-md cursor-pointer min-w-[100px] flex-shrink-0 {bg_class} {border_class} {text_class} {z_class} -mb-px select-none"
        );

        rsx! {
            div {
                key: "{tab.id}",
                class: "{class_name}",
                onclick: move |_| activate_tab(tab.id),
                "{tab.title}",
                button {
                    class: "ml-2 text-gray-800 hover:text-red-500",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        close_tab(tab.id);
                    },
                    "×"
                }
            }
        }
    })
    .collect::<Vec<_>>();

    rsx! {
        div {
            class: "relative px-2 border-b border-gray-300 flex items-end space-x-1",
            div {
                class: "flex overflow-x-auto whitespace-nowrap no-scrollbar",
                // Fix here: spread vector with into_iter()
                {rendered_tabs.into_iter()}
            }
            button {
                class: "w-6 h-6 mb-[1px] bg-gray-200 hover:bg-gray-300 rounded-full text-gray-700 text-lg flex items-center justify-center flex-shrink-0",
                onclick: add_tab,
                "+"
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Tab {
    id: usize,
    title: String,
    active: bool,
}
