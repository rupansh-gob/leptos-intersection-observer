use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_debounce_fn;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/intersection-observer-bug.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let data = create_rw_signal(vec![0usize]);
    let fetch_next_data = create_action(move |()| async move {
        // Do async stuff
        let idx = data.with_untracked(|d| *d.last().unwrap());
        data.update(|d| d.extend(idx + 1..idx + 1 + 20));
    });
    let _next_data = use_debounce_fn(
        move || {
            if !fetch_next_data.pending().get_untracked() {
                fetch_next_data.dispatch(())
            }
        },
        500.0,
    );
    let _scroll_root = create_node_ref::<html::Div>();
    let current_idx = create_rw_signal(0usize);
    let virtual_data = move || {
        let cur_idx = current_idx().max(15 / 2) - (15 / 2);
        data.with(|d| d[cur_idx..].iter().copied().take(15).collect::<Vec<_>>())
    };

    view! {
        <div
            _ref=_scroll_root
            class="snap-mandatory snap-y overflow-y-scroll h-dvh w-dvw bg-black"
            style:scroll-snap-points-y="repeat(100vh)"
        >
            <For
                each=virtual_data
                key=|idx| *idx
                children=move |idx| {
                    let _ref = create_node_ref::<html::Div>();

                    #[cfg(feature = "hydrate")]
                    {
                        use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};
                        let next_data = _next_data.clone();
                        use_intersection_observer_with_options(_ref, move |entry, _| {
                            if entry.first().filter(|e| e.is_intersecting()).is_none() {
                                return;
                            }
                            if data.with_untracked(|d| d.len()).saturating_sub(idx) <= 10 {
                                next_data();
                            }
                            current_idx.set(idx);
                        },
                        UseIntersectionObserverOptions::default()
                            .thresholds(vec![0.83])
                            .root(Some(_scroll_root))
                        );
                    }

                    view! {
                        <div
                            _ref=_ref
                            class="snap-always snap-end w-full h-full"
                        >
                            <div class="flex justify-center items-center h-full w-full text-white text-4xl">
                                {idx}
                            </div>
                        </div>
                    }
                }
            >
            </For>
        </div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
