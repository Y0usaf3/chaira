use leptos::prelude::*;

#[component]
pub fn FilteredInput(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    #[prop(into)] value: Signal<String>,
    set_value: WriteSignal<String>,
    #[prop(into)] filter: Callback<String, String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1">
            <label class="text-sm font-semibold text-slate-700">{label}</label>
            <div class="pixel-input--wrapper p-4">
                <input
                    type="text"
                    class="placeholder:text-slate-400 focus:outline-none bg-transparent w-full"
                    placeholder=placeholder
                    on:input=move |ev| {
                        let raw_val = event_target_value(&ev);
                        let filtered_val = filter.run(raw_val);
                        set_value.set(filtered_val);
                    }
                    prop:value=value
                />
            </div>
        </div>
    }
}
