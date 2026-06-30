use leptos::prelude::*;
use models::Field;

use crate::components::{icon::field_icon, Icon};

#[component]
pub fn Field(
    field: Field,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
) -> impl IntoView {
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
        <div
            class="min-w-[100px] pl-2 h-full flex items-center border-r-[2px] border-black select-none"
            style:width=move || format!("{}px", local_width.get())
        >
            <Icon
                icon_type=icon_type
                class="w-[16px] h-auto fill-black mr-2 shrink-0"
                fill="#000000"
            />
            <p>{field.name}</p>
            <div
                node_ref=handle_ref
                class="ml-auto w-[4px] h-full cursor-col-resize bg-black hover:bg-neutral-600 shrink-0 touch-none self-stretch"
                on:pointerdown=on_pointerdown
                on:pointermove=on_pointermove
                on:pointerup=on_pointerup
            ></div>
        </div>
    }
}
