use super::ui::dropdown_menu::*;
use crate::components::ui::separator::*;
use icons::{Annoyed, Calendar, Clock, Hash, Mail, Shield, User as UserIcon};
use leptos::{prelude::*, tachys::view::keyed::SerializableKey};

#[server]
pub async fn get_user() -> Result<charac::models::User, ServerFnError> {
    let service = crate::get_authenticated_service().await?;
    Ok(service.user)
}

#[component]
pub fn SideNav() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| async move { get_user().await });

    view! {
    <aside class="sticky top-0 h-screen w-20 border-r bg-background p-4 flex flex-col items-center justify-between shrink-0">            <div class="flex flex-col items-center gap-2">
                   <Annoyed class="size-10 text-[var(--foreground)] border p-1 rounded" />
                   <span class="hidden sm:block font-bold text-[10px] uppercase tracking-tighter">"CHARA"</span>
               </div>
               <DropdownMenu align=DropdownMenuAlign::EndOuter>
                   <DropdownMenuTrigger class="p-0 bg-transparent border-0 outline-none">
                       <div class="flex flex-col items-center">
                           <Suspense fallback=move || view! {
                               <div class="size-10 rounded-lg bg-secondary animate-pulse" />
                           }>
                               <span class="flex overflow-hidden relative rounded-lg size-10 shrink-0 bg-secondary justify-center items-center font-bold text-sm hover:ring-2 ring-primary transition-all cursor-pointer">
                                   {move || user_resource.get().and_then(|res| res.ok()).map(|u| {
                                       u.first_name.chars().next().unwrap_or('?')
                                   })}
                               </span>
                           </Suspense>
                       </div>
                   </DropdownMenuTrigger>
                   <DropdownMenuContent class="w-[260px] mr-4">
                       <Suspense fallback=move || view! { <DropdownMenuLabel>"Loading User..."</DropdownMenuLabel> }>
                           {move || user_resource.get().map(|res| {
                               match res {
                                   Ok(user) => view! {
                                       <DropdownMenuLabel>"User Profile"</DropdownMenuLabel>
                                       <DropdownMenuItem class="flex gap-2">
                                           <UserIcon class="size-4 opacity-70" />
                                           <span>{user.first_name} " " {user.last_name}</span>
                                       </DropdownMenuItem>
                                       <DropdownMenuItem class="flex gap-2">
                                           <Mail class="size-4 opacity-70" />
                                           <span>{user.email}</span>
                                       </DropdownMenuItem>
                                       <DropdownMenuItem class="flex gap-2">
                                           <Shield class="size-4 opacity-70" />
                                           <span class="capitalize">{user.role}</span>
                                       </DropdownMenuItem>

                                       <Separator class="my-1" />

                                       <DropdownMenuLabel>"System Data"</DropdownMenuLabel>
                                       <DropdownMenuItem class="flex gap-2">
                                           <Hash class="size-4 opacity-70" />
                                           <span class="text-xs font-mono">
                                               {format!("{:?}",user.id.unwrap().0.key)}
                                           </span>
                                       </DropdownMenuItem>

                                       <DropdownMenuItem class="flex gap-2">
                                           <Calendar class="size-4 opacity-70" />
                                           <span class="text-xs text-muted-foreground">
                                               "Created: " {user.created_at.map(|d| d.to_string()).unwrap_or_else(|| "N/A".into())}
                                           </span>
                                       </DropdownMenuItem>

                                       <DropdownMenuItem class="flex gap-2">
                                           <Clock class="size-4 opacity-70" />
                                           <span class="text-xs text-muted-foreground">
                                               "Updated: " {user.updated_at.map(|d| d.to_string()).unwrap_or_else(|| "N/A".into())}
                                           </span>
                                       </DropdownMenuItem>
                                   }.into_any(),

                                   Err(_) => view! {
                                       <DropdownMenuLabel class="text-destructive">"Failed to load user"</DropdownMenuLabel>
                                   }.into_any()
                               }
                           })}
                       </Suspense>

                       <Separator class="my-1" />

                       <DropdownMenuGroup>
                           <DropdownMenuItem>
                               <DropdownMenuAction>"Sign Out"</DropdownMenuAction>
                           </DropdownMenuItem>
                       </DropdownMenuGroup>
                   </DropdownMenuContent>
               </DropdownMenu>
           </aside>
       }
}
