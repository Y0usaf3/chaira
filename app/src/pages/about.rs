use leptos::prelude::*;

use crate::components::Footer;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="w-screen flex flex-col min-h-screen items-center bg-slate-50 h-full">

            <img src="/chaira.png" class="w-[480px] h-auto object-contain pixelated m-[20px]" />

            <div class="align-center pixel-corners--wrapper px-20 py-15 mt-auto mb-8 max-w-[70%]">
                <h1 class="text-xl pb-2">"What is Chaira all about ?"</h1>
                <p>
                    "Chaira is being made for the one goal... replacing Airtable, it will feature automations, special types for handling hackclubbers data and also let those to see their own data securely by using hackclub auth directly from the backend.."
                </p>
                <p>
                    "Hack Club is a nonprofit that helps teenagers like me and you (if you're still a teen) to build awesome stuff with software and hardware, and teenagers can organise their own event too-- or should i say, YouShipWeShips."
                </p>
                <p>
                    "Now let me introduce you to what a YSWS is (even if there is an 80% chance you already know). A YSWS is an event where you basically ship a project. It could be software like a CLI tool or an app, or hardware, like a robot, macropads, or even guns /silly (yeah no, probably not a gun lol). Then we ship you a reward—think nerdy circuit boards, your favorite game, laptops, etc."
                </p>
                <p>
                    "But as you know (wait, why didn't you know?!), Hack Club has to be transparent. Every expense has to be public. So what did they do to solve that? They made their own virtual bank, HCB (which stands for Hack Club Bank). Here you can see the current amount of money they have, where it gets transferred, how much money an event is spending, and all that stuff. But you know what else? They also have to store data about Hack Clubbers. OF COURSE! Hack Club HAS to know how many hours were spent in events so they can calculate those juicy weighted hours."
                </p>
                <p>
                    "And you know where they store this data? Well, I will tell you. It's in AIRTABLE. YES, it's so surprising. A nonprofit is actually paying a company to store the data of several events... that's where I come in..."
                </p>
                <p>".........."</p>
                <p>
                    "What ? i just want to replace airtable in hackclubs infra, whats wrong with that! Maybe the developpement is going pretty slow, but im putting all my heart on this project, so one day i would finally be proud of something i made.."
                </p>
                <p>
                    "erm, its rare that someone actually reads this, so thank you for reading! i really appreaciate :3"
                </p>
            </div>

            <Footer />
        </div>
    }
}
