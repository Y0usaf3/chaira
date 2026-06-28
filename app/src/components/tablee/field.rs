use crate::components::{Icon, IconType};
use leptos::prelude::*;
use models::Field as Fild;
use models::{FieldConfig, TextConfig};

#[component]
pub fn Field(
    field: Fild,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
) -> impl IntoView {
    let name = field.name.clone();
    let icon_type = field_icon(&field.config);

    let handle_ref = NodeRef::<leptos::html::Div>::new();
    let (is_resizing, set_is_resizing) = signal(false);
    let start_x = RwSignal::new(0.0);
    let local_width = RwSignal::new(width.get_untracked());

    Effect::new(move |_| {
        if !is_resizing.get_untracked() {
            local_width.set(width.get());
        }
    });

    let on_pointerdown = move |ev: leptos::ev::PointerEvent| {
        ev.prevent_default();
        set_is_resizing.set(true);
        start_x.set(ev.client_x() as f64);
        let _ = handle_ref
            .get()
            .map(|el| el.set_pointer_capture(ev.pointer_id()));
    };

    let on_pointermove = move |ev: leptos::ev::PointerEvent| {
        if is_resizing.get_untracked() {
            let dx = ev.client_x() as f64 - start_x.get();
            let current = local_width.get_untracked();
            let new_w = (current + dx).max(60.0);
            let stepped = ((new_w / 2.0).round() * 2.0) as f64;
            local_width.set(stepped);
            if let Some(ref cb) = on_resize {
                cb.run(stepped);
            }
            start_x.set(ev.client_x() as f64);
        }
    };

    let on_pointerup = move |ev: leptos::ev::PointerEvent| {
        if is_resizing.get_untracked() {
            set_is_resizing.set(false);
            let _ = handle_ref
                .get()
                .map(|el| el.release_pointer_capture(ev.pointer_id()));
        }
    };

    view! {
        <div class="flex items-center border-r-2 border-b-2 border-black bg-white shrink-0 select-none min-h-[32px] w-full">
            {if let Some(icon_type) = icon_type {
                view! { <Icon icon_type class="w-[14px] h-[14px] shrink-0 ml-1 mr-2" /> }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }} <span class="text-base font-semibold text-black truncate px-1 py-1">{name}</span>
            <div
                node_ref=handle_ref
                class="ml-auto w-[4px] h-full cursor-col-resize hover:bg-black active:bg-black shrink-0 touch-none self-stretch"
                style:background-color=move || {
                    if is_resizing.get() { "#000000" } else { "transparent" }
                }
                on:pointerdown=on_pointerdown
                on:pointermove=on_pointermove
                on:pointerup=on_pointerup
            ></div>
        </div>
    }
}
