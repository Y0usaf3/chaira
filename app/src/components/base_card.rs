use crate::components::Button;
use crate::name_to_hex_color;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use models::Base;
use models::ToSql;

#[component]
pub fn BaseCard(base: Base) -> impl IntoView {
    let color = name_to_hex_color(&base.name);
    let created_date = base
        .created_at
        .map(|d| d.to_string().chars().take(10).collect::<String>())
        .unwrap_or_else(|| "Unknown date".to_string());
    let path = match base.id {
        Some(id) => format!("/base/{}", id.0.key.to_sql()),
        _ => format!("/base/404"),
    };
    let naviguate = use_navigate();
    let naviguate_to_its_base = move |_| {
        naviguate(path.as_str(), NavigateOptions::default());
    };

    view! {
        <div class="flex flex-col h-full min-h-[180px] pixel-corners--wrapper bg-white !w-full">

            <div
                class="h-[80px] border-b-[3px] border-black flex-none"
                style=format!("background-color: {};", color)
            ></div>
            <div class="flex flex-col flex-grow p-4 gap-3">
                <div class="flex-grow">
                    <h3 class="text-xl font-bold truncate text-black">{base.name}</h3>
                </div>

                <div class="flex justify-end items-end">
                    <p class="text-xl text-gray-500 uppercase small-font mr-auto">{created_date}</p>
                    <Button class="w-full sm:w-auto font-bold" on:click=naviguate_to_its_base>
                        "Open"
                    </Button>
                </div>

            </div>
        </div>
    }
}
