use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0">
                <div class="flex h-14 w-full">
                    <img
                        src="/small_chaira.png"
                        class="h-auto w-[40px] object-contain pixelated ml-auto mt-auto"
                        alt="Chaira"
                    />
                </div>
            </div>

            <div class="flex flex-1 flex-col overflow-hidden">
                <div class="h-14 flex-shrink-0">
                    <div class="flex h-full w-full items-center justify-between"></div>
                </div>
                <main class="pixel-corners-wrapper flex-1 overflow-hidden p-6 mb-[-3px] mr-[-3px]">
                    <div class="overflow-y-auto">
                        <p>"imagine some bases here"</p>
                        <p>"and other stuff here"</p>
                        <p>"ooh look a notification"</p>
                        <p>"its from who ??"</p>
                        <p>"who tf is rael?!"</p>
                    </div>
                </main>
            </div>
        </div>
    }
}
