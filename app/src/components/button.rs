use leptos::prelude::*;
use leptos_ui::variants;

variants! {
    Button {
       base: "pixel-corners--wrapper inline-flex items-center justify-center gap-2 text-lg font-medium shrink-0 outline-none",
        variants: {
            variant: {
                Default: "bg-primary text-primary-foreground hover:bg-primary/90",
            },
            size: {
                Default: "h-12 px-4 py-2 pb-1",
            }
        },
        component: {
            element: button,
            support_href: true,
            support_aria_current: true
        }
    }
}
