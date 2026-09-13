// Reusable presentational components go here.

use leptos::prelude::*;

#[component]
pub fn TitleAndDelete(title: String) -> impl IntoView {
    view! {
        <nav class="level">
            <div class="level-left">
                <div class="level-item">
                    <h2 class="title">{title}</h2>
                </div>
            </div>
            <div class="level-right">
                <div class="level-item">
                    <button class="delete"></button>
                </div>
            </div>
        </nav>
    }
}
