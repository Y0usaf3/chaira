use crate::components::{
    hooks::use_theme_mode::ThemeMode,
    sidenav::SideNav,
    ui::{
        breadcrumb::{
            Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbSeparator,
        },
        button::{Button, ButtonSize, ButtonVariant},
        empty::*,
        theme_toggle::ThemeToggle,
    },
};
use components::{BaseBox, CreateBaseDialog};
use icons::{ArrowUpRight, FolderCode, Lock, Plus};
use leptos::prelude::*;
use server::{create_base, create_token, get_user_bases};

mod components;
pub mod server;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let theme = ThemeMode::init();

    let (refresh_count, set_refresh_count) = signal(0);
    let bases = Resource::new(
        move || refresh_count.get(),
        |_| async move { get_user_bases().await },
    );

    Effect::new(move || {
        if let Some(Err(_)) = bases.get() {
            window().location().assign("/").unwrap();
        }
    });

    let create_base_action = Action::new(|name: &String| {
        let name = name.clone();
        async move { create_base(name).await }
    });

    Effect::new(move |_| {
        if create_base_action.value().get().is_some() {
            set_refresh_count.update(|n| *n += 1);
        }
    });

    let create_message = move || {
        create_base_action.value().get().map(|res| {
            if let Err(e) = res {
                view! { <p class="text-destructive text-sm font-medium">{e.to_string()}</p> }
                    .into_any()
            } else {
                view! { <p class="text-sm text-muted-foreground">"Base created successfully!"</p> }
                    .into_any()
            }
        })
    };
    let create_token_action = Action::new(|_: &()| async move { create_token().await });

    view! {
        <div
            class:dark=move || theme.is_dark()
            class="flex min-h-screen bg-[var(--background)] text-[var(--foreground)]"
        >
            <SideNav />
            <div class="flex-1 relative p-8">
                <div class="absolute top-4 right-4 p-2">
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        on:click=move |_| {
                            create_token_action.dispatch(());
                        }
                        attr:disabled=move || create_token_action.pending().get()
                    >
                        {move || {
                            if create_token_action.pending().get() {
                                "Creating..."
                            } else {
                                "Generate API Token"
                            }
                        }}
                    </Button>
                    <ThemeToggle />
                </div>

                <div class="flex flex-col gap-6 h-full w-full">
                    <Breadcrumb>
                        <BreadcrumbList>
                            <BreadcrumbItem>
                                <BreadcrumbLink attr:href="/dashboard">"Dashboard"</BreadcrumbLink>
                            </BreadcrumbItem>

                            <BreadcrumbSeparator />
                        </BreadcrumbList>
                    </Breadcrumb>

                    {move || {
                        create_token_action
                            .value()
                            .get()
                            .map(|res| {
                                match res {
                                    Ok(token) => {
                                        view! {
                                            <div class="p-4 border border-primary/20 bg-primary/5 rounded-lg flex flex-col gap-2">
                                                <span class="text-xs font-bold uppercase tracking-wider opacity-60">
                                                    "Your New API Token"
                                                </span>
                                                <div class="flex gap-2 items-center">
                                                    <code class="bg-background p-2 rounded border flex-1 font-mono text-sm">
                                                        {token}
                                                    </code>
                                                    <p class="text-xs text-muted-foreground italic">
                                                        "Make sure to copy this now. You won't see it again!"
                                                    </p>
                                                </div>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                    Err(e) => {
                                        view! { <p class="text-destructive">{e.to_string()}</p> }
                                            .into_any()
                                    }
                                }
                            })
                    }}

                    <div class="flex gap-4 justify-end">
                        <CreateBaseDialog
                            title=move || {
                                if create_base_action.pending().get() {
                                    view! { <Lock /> }.into_any()
                                } else {
                                    view! { <Plus /> }.into_any()
                                }
                            }
                            create_action=create_base_action
                        />
                    </div>

                    <Suspense>
                        {move || Suspend::new(async move {
                            match bases.get() {
                                Some(Ok(list)) if list.is_empty() => {
                                    view! {
                                        <Empty class="flex-1 flex items-center justify-center">
                                            <EmptyHeader>
                                                <EmptyMedia variant=EmptyMediaVariant::Icon>
                                                    <FolderCode />
                                                </EmptyMedia>
                                                <EmptyTitle>"No Base Yet"</EmptyTitle>
                                                <EmptyDescription>
                                                    "You haven't created any bases yet. Get started by creating your first base! :3"
                                                </EmptyDescription>
                                                <EmptyDescription>
                                                    {move || create_message()}
                                                </EmptyDescription>
                                            </EmptyHeader>

                                            <EmptyContent>
                                                <div class="flex gap-2">
                                                    <CreateBaseDialog
                                                        title="Create".into_any()
                                                        create_action=create_base_action
                                                    />

                                                </div>

                                                <Button
                                                    variant=ButtonVariant::Link
                                                    size=ButtonSize::Sm
                                                    class="text-muted-foreground"
                                                >
                                                    <a href="#" class="flex gap-1 items-center">
                                                        <span>"Learn More"</span>
                                                        <ArrowUpRight />
                                                    </a>
                                                </Button>

                                            </EmptyContent>
                                        </Empty>
                                    }
                                        .into_any()
                                }
                                Some(Ok(list)) => {
                                    view! {
                                        <div class="grid grid-cols-3 gap-3">
                                            {list
                                                .into_iter()
                                                .map(|base| {
                                                    view! { <BaseBox base=base /> }
                                                })
                                                .collect_view()}
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(Err(_)) => {
                                    view! {
                                        <p class="text-red-500">
                                            "Unauthentified ! you silly goober"
                                        </p>
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! { <p class="text-red-500">"Unknown error"</p> }.into_any()
                                }
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
