use crate::components::Button;
use crate::name_to_hex_color;
use leptos::prelude::*;
use models::Base;

#[component]
pub fn BaseCard(base: Base) -> impl IntoView {
    let color = name_to_hex_color(&base.name);
    let created_date = base
        .created_at
        .map(|d| d.to_string().chars().take(10).collect::<String>())
        .unwrap_or_else(|| "Unknown date".to_string());

    view! {
        <div class="flex flex-col h-full min-h-[180px] pixel-corners--wrapper bg-white">

            <div
                class="h-[64px] border-b-2 border-black flex-none"
                style=format!("background-color: {};", color)
            ></div>
            <div class="flex flex-col flex-grow p-4 gap-3">
                <div class="flex-grow">
                    <h3 class="text-xl font-bold truncate text-black">{base.name}</h3>
                    <p class="text-xl text-gray-500 mt-1 uppercase small-font">
                        "Created: " {created_date}
                    </p>
                </div>

                <div class="mt-2 flex justify-end">
                    <Button class="w-full sm:w-auto font-bold">"Open"</Button>
                </div>

            </div>
        </div>
    }
}
