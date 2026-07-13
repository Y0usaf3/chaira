use leptos::prelude::*;
use leptos_ui::variants;

variants! {
    Button {
       base: "pixel-corners--wrapper inline-flex items-center justify-center gap-2 text-lg font-medium shrink-0 outline-none",
        variants: {
            variant: {
                Default: "bg-black text-white leading-none",
            },
            size: {
                Default: "h-12 px-6 py-4",
            }
        },
        component: {
            element: button,
            support_href: true,
            support_aria_current: true
        }
    }
}
