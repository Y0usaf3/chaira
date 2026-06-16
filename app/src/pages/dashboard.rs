use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[server]
pub async fn get_user_bases() -> Result<Vec<UserBase>, ServerFnError> {
    use surrealdb::types::ToSql;
    let service = crate::get_authenticated_service().await?;

    let bases = service
        .list_bases()
        .await
        .map_err(|e| ServerFnError::new(format!("Listing Bases failed: {e:?}")))?;

    let user_bases = bases
        .into_iter()
        .map(|b| UserBase {
            name: b.name,
            owner_name: b.owner.0.key.to_sql(),
            id: b.id.unwrap().0.key.to_sql(),
        })
        .collect();
    Ok(user_bases)
}

#[server]
pub async fn create_base(name: String) -> Result<UserBase, ServerFnError> {
    let service = crate::get_authenticated_service().await?;
    let base = service
        .create_base(name)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(UserBase {
        name: base.name,
        owner_name: format!("{:?}", base.owner.0.key),
        id: base
            .id
            .map(|id| format!("{:?}", id.0.key))
            .unwrap_or_default(),
    })
}

#[component]
pub fn DashboardPage() -> impl IntoView {
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
    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0 flex flex-col h-full">
                <div class="flex h-14 w-full">
                    <img
                        src="/image/small_chaira.png"
                        class="h-auto w-[40px] object-contain pixelated ml-auto mt-auto"
                        alt="Chaira"
                    />
                </div>

                <div class="w-full flex justify-center mt-auto py-[4.5]">
                    <button class="pixel-corners-pfp bg-black w-[32px] h-[32px] flex items-center justify-center">
                        <img src="/svg/plus.svg" class="w-[16px] h-[16px] pixelated fill-white" />
                    </button>
                </div>
            </div>

            <div class="flex flex-1 flex-col overflow-hidden">
                <div class="h-14 flex-shrink-0">
                    <div class="flex h-full w-full items-center justify-end px-[4.5]">
                        <img
                            src="https://i.pinimg.com/736x/7c/41/86/7c41866499a79bca61ecf049973f5d76.jpg"
                            class="h-[40px] w-[40px] object-cover pixelated my-auto pixel-corners-pfp grayscale-75"
                        />
                    </div>
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
