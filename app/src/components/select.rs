use leptos::portal::Portal;
use leptos::prelude::*;

use crate::components::icon::{Icon, IconType};

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Select(
    #[prop(into)] options: Vec<SelectOption>,
    getter: ReadSignal<String>,
    setter: WriteSignal<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (top, set_top) = signal(0.0_f64);
    let (left, set_left) = signal(0.0_f64);
    let (width, set_width) = signal(0.0_f64);

    let btn_ref = NodeRef::<leptos::html::Button>::new();

    if getter.get_untracked().is_empty()
        && let Some(first) = options.first()
    {
        setter.set(first.value.clone());
    }

    let toggle = move |ev: leptos::ev::MouseEvent| {
        if !is_open.get_untracked() {
            #[cfg(target_arch = "wasm32")]
            {
                let el = event_target::<web_sys::HtmlElement>(&ev);
                let rect = el.get_bounding_client_rect();
                set_top.set(rect.bottom());
                set_left.set(rect.left());
                set_width.set(rect.width());
            }
        }
        set_is_open.update(|open| *open = !*open);
    };

    let options_store = StoredValue::new(options);
    let label = move || {
        let current = getter.get();
        options_store
            .get_value()
            .iter()
            .find(|o| o.value == current)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| "Select an option...".to_string())
    };

    let merged_class = format!(
        "pixel-corners--wrapper !w-full !h-[49px] p-4 inline-flex items-center justify-between text-sm {}",
        class
    );
    let rotate_class = move || if is_open.get() {"rotate-180"} else {""};

    view! {
        <div class="relative w-full">
            <button type="button" node_ref=btn_ref on:click=toggle class=merged_class>
                <span class="text-sm truncate flex items-center gap-2 pointer-events-none">
                    {label}
                </span>
                <Icon
                    icon_type=IconType::ChevronDown
                    class="text-muted-foreground w-[20px] h-auto pointer-events-none"
                    extern_class=rotate_class
                    fill="#000000"
                />
            </button>

            <Portal>
                <div
                    class="bg-white pixel-corners--wrapper z-[9999] overflow-visible !w-[173px]"
                    style:position="fixed"
                    style:top=move || { format!("{}px", top.get() + 8.0) }
                    style:left=move || { format!("{}px", left.get()) }
                    style:width=move || format!("{}px", width.get())
                    style:display=move || if is_open.get() { "block" } else { "none" }
                >
                    <For
                        each=move || options_store.get_value()
                        key=|opt| opt.value.clone()
                        children=move |opt| {
                            let val = opt.value.clone();
                            let val_clone_i_guess = val.clone();
                            let is_selected = move || getter.get() == val_clone_i_guess;
                            let is_not_fucking_selected = move || getter.get() != opt.value;
                            let selected_for_class = is_selected.clone();

                            view! {
                                <div
                                    on:click=move |_| {
                                        setter.set(val.clone());
                                        set_is_open.set(false);
                                    }
                                    class="px-4 py-3 text-black cursor-pointer flex justify-between items-center"
                                    class:bg-black=selected_for_class.clone()
                                    class:text-white=selected_for_class.clone()
                                    class:hover:bg-slate-50=is_not_fucking_selected
                                >
                                    <span>{opt.label.clone()}</span>
                                    {move || {
                                        is_selected()
                                            .then(|| {
                                                view! {
                                                    <Icon
                                                        icon_type=IconType::Check
                                                        class="w-[18px] h-auto"
                                                        fill="#FFFFFF"
                                                    />
                                                }
                                            })
                                    }}
                                </div>
                            }
                        }
                    />
                </div>
            </Portal>
        </div>
    }
}
