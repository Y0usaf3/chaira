use leptos::html::Input;
use leptos::prelude::*;

#[component]
pub fn FilteredInput(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    #[prop(into)] value: Signal<String>,
    set_value: WriteSignal<String>,
    #[prop(into)] filter: Callback<String, String>,
    #[prop(into)] autofocus: bool,
) -> impl IntoView {
    let input_ref = NodeRef::<Input>::new();
    if autofocus {
        Effect::new(move |_| {
            if let Some(el) = input_ref.get() {
                request_animation_frame(move || {
                    let _ = el.focus();
                });
            }
        });
    }
    view! {
        <div class="flex flex-col gap-1">
            <label class="text-sm font-semibold text-slate-700">{label}</label>
            <div class="pixel-input--wrapper p-4">
                <input
                    type="text"
                    node_ref=input_ref
                    class="placeholder:text-slate-400 focus:outline-none bg-transparent w-full"
                    placeholder=placeholder
                    on:input=move |ev| {
                        let raw_val = event_target_value(&ev);
                        let filtered_val = filter.run(raw_val);
                        set_value.set(filtered_val);
                    }
                    prop:value=value
                    autofocus=autofocus
                />
            </div>
        </div>
    }
}

#[component]
pub fn ValidatedInput(
    #[prop(into)] value: Signal<String>,
    set_value: WriteSignal<String>,
    #[prop(into)] validate: Callback<String, Result<(), String>>,
    #[prop(into)] placeholder: String,
) -> impl IntoView {
    let (has_error, set_has_error) = signal(false);
    let input_ref = NodeRef::<Input>::new();

    view! {
        <div class="relative w-full">
            <input
                type="text"
                node_ref=input_ref
                class="w-full bg-transparent outline-none placeholder:text-slate-400"
                placeholder=placeholder
                prop:value=value
                on:input=move |ev| {
                    set_value.set(event_target_value(&ev));
                    set_has_error.set(false);
                }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        let val = value.get();
                        set_has_error.set(validate.run(val).is_err());
                    }
                }
                on:blur=move |_| {
                    let val = value.get();
                    if !val.is_empty() {
                        set_has_error.set(validate.run(val).is_err());
                    }
                }
            />
            <div
                class="absolute bottom-0 right-0 w-1.5 h-1.5"
                class:opacity-100=move || has_error.get()
                class:opacity-0=move || !has_error.get()
                style="background-color: #ef4444"
            ></div>
        </div>
    }
}
