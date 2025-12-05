use dioxus::prelude::*;
use dioxus_core::{ScopeState, EventHandler}; // Correctly import ScopeState and EventHandler
use dioxus_signals::{Signal, ReadableExt, WritableExt}; // Keep this import
use solfunmeme_loader::{AnyMeme, MemeSource};
use std::rc::Rc;
use log::error; // Import log::error

// Wrapper for Box<dyn AnyMeme> to implement PartialEq and Clone
pub struct AnyMemeWrapper(pub Box<dyn AnyMeme>);

impl AnyMemeWrapper {
    // Helper to access the inner AnyMeme trait object
    pub fn inner(&self) -> &dyn AnyMeme {
        self.0.as_ref()
    }
}

// Manual implementation of PartialEq for AnyMemeWrapper
impl PartialEq for AnyMemeWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.equals(other.0.as_ref()) // Use the equals method from AnyMeme trait
    }
}

// Manual implementation of Clone for AnyMemeWrapper
impl Clone for AnyMemeWrapper {
    fn clone(&self) -> Self {
        AnyMemeWrapper(self.0.box_clone()) // Use the box_clone method from AnyMeme trait
    }
}


// Wrapper for Rc<dyn MemeSource> to implement PartialEq and Clone
pub struct MemeSourceWrapper(pub Rc<dyn MemeSource>);

impl MemeSourceWrapper {
    // Helper to access the inner MemeSource trait object
    pub fn inner(&self) -> &dyn MemeSource {
        self.0.as_ref()
    }
}

impl PartialEq for MemeSourceWrapper {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Clone for MemeSourceWrapper {
    fn clone(&self) -> Self {
        MemeSourceWrapper(self.0.clone())
    }
}


// Define a new inner component to handle the actual UI logic
#[component]
fn MemeManagementInner(cx: ScopeState, meme_source: MemeSourceWrapper) -> Element {
    let selected_category = use_signal(|| "Component Memes".to_string());
    let selected_meme_any = use_signal(|| None::<AnyMemeWrapper>);
    let show_meme_details = use_signal(|| false);
    let search_query = use_signal(|| String::new());

    // Retrieve all memes as AnyMeme trait objects using use_ref
    let all_memes: Signal<Vec<AnyMemeWrapper>> = use_signal(|| {
        meme_source.inner().get_all_memes()
            .unwrap_or_else(|e| {
                error!("Failed to get all memes from source: {:?}", e);
                vec![]
            })
            .into_iter()
            .map(AnyMemeWrapper)
            .collect()
    });

    let filtered_memes: Memo<Vec<AnyMemeWrapper>> = use_memo(move || {
        all_memes.read().iter()
            .filter(|m_any| m_any.inner().category_name() == selected_category.read().as_str())
            .filter(|m_any| {
                if search_query.read().is_empty() {
                    true
                } else {
                    let query = search_query.read().to_lowercase();
                    m_any.inner().name().to_lowercase().contains(&query) || m_any.inner().description().to_lowercase().contains(&query)
                    || m_any.inner().tags().iter().any(|tag| tag.to_lowercase().contains(&query))
                }
            })
            .cloned() // Clone the AnyMemeWrapper, which clones the inner Box<dyn AnyMeme>
            .collect()
    });

    // Manually trigger updates for filtered_memes when dependencies change
    use_effect(move || {
        let new_filtered_memes: Vec<AnyMemeWrapper> = all_memes.read().iter()
            .filter(|m_any| m_any.inner().category_name() == selected_category.read().as_str())
            .filter(|m_any| {
                if search_query.read().is_empty() {
                    true
                } else {
                    let query = search_query.read().to_lowercase();
                    m_any.inner().name().to_lowercase().contains(&query) || m_any.inner().description().to_lowercase().contains(&query)
                    || m_any.inner().tags().iter().any(|tag| tag.to_lowercase().contains(&query))
                }
            })
            .cloned()
            .collect();
        filtered_memes.set(new_filtered_memes);
    });

    rsx! {
        div {
            // class: "{Styles::section()}", // Styles need to be addressed
            h2 { /*class: "{Styles::h2()}",*/ "🎭 Meme Management Toolbox" } // Styles need to be addressed
            p { /*class: "{Styles::p()}",*/ "Explore and manage different types of memes for your SolFunMeme application." }

            div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6 mt-6",
                // Meme Categories Sidebar
                div { class: "lg::col-span-1",
                    div { class: "bg-white dark:bg-gray-800 shadow-lg rounded-lg p-4",
                        h3 { class: "text-lg font-semibold mb-4 text-gray-900 dark:text-white", "Meme Categories" }

                        // Search Bar
                        div { class: "mb-4",
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white",
                                placeholder: "Search memes...",
                                value: "{search_query.read()}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }

                        // Categories are now hardcoded as strings or retrieved from MemeSource metadata if available
                        for category_str in ["Component Memes", "Workflow Memes", "Wikidata Memes", "Crypto Memes", "Lean Memes", "Fun Memes"] {
                            button {
                                class: format!(
                                    "w-full text-left p-3 mb-2 rounded-lg transition-colors flex items-center gap-2 {}",
                                    if selected_category.read().as_str() == category_str {
                                        "bg-blue-500 text-white"
                                    } else {
                                        "bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white"
                                    }
                                ),
                                onclick: move |_| selected_category.set(category_str.to_string()),
                                span { class: "text-lg", "{get_emoji_for_category_string(category_str)}" }
                                span { "{category_str}" }
                            }
                        }
                    }
                }

                div { class: "lg::col-span-3",
                    div { class: "bg-white dark:bg-gray-800 shadow-lg rounded-lg p-6",
                        div { class: "flex justify-between items-center mb-4",
                            h3 { class: "text-xl font-semibold text-gray-900 dark:text-white",
                                "{get_emoji_for_category_string(&selected_category.read())} {selected_category.read()}"
                            }
                            div { class: "flex gap-2",
                                button {
                                    class: "bg-green-500 text-white px-4 py-2 rounded-lg hover:bg-green-600 transition-colors",
                                    onclick: move |_| {
                                        // Add new meme functionality
                                    },
                                    "➕ Add Meme"
                                }
                                button {
                                    class: "bg-purple-500 text-white px-4 py-2 rounded-lg hover:bg-purple-600 transition-colors",
                                    onclick: move |_| {
                                        // Import memes functionality
                                    },
                                    "📥 Import"
                                }
                            }
                        }

                        div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4",
                            for meme_wrapper in filtered_memes.read().iter() {
                                MemeCard {
                                    meme: meme_wrapper.clone(),
                                    on_select: move |selected_meme_wrapper: AnyMemeWrapper| {
                                        selected_meme_any.set(Some(selected_meme_wrapper));
                                        show_meme_details.set(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Meme Details Modal
            if *show_meme_details.read() {
                if let Some(meme_wrapper) = selected_meme_any.read().as_ref() {
                    MemeDetailsModal {
                        meme: meme_wrapper.clone(),
                        on_close: move |_| {
                            show_meme_details.set(false);
                            selected_meme_any.set(None);
                        }
                    }
                }
            }
        }
    }
}

// The outer MemeManagement function now just calls the inner component
#[component]
pub fn MemeManagement(cx: ScopeState, meme_source: MemeSourceWrapper) -> Element {
    rsx! { MemeManagementInner { meme_source: meme_source.clone() } }
}

#[component]
fn MemeCard(cx: ScopeState, meme: AnyMemeWrapper, on_select: EventHandler<AnyMemeWrapper>) -> Element {
    let meme1 = meme.clone();
    let meme2 = meme.clone();

    rsx! {
        div {
            class: "border border-gray-200 dark:border-gray-600 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer bg-white dark:bg-gray-700",
            onclick: move |_| on_select.call(meme1.clone()),

            div { class: "flex items-center gap-3 mb-3",
                span { class: "text-2xl", "{meme.inner().emoji()}" }
                div {
                    h4 { class: "font-medium text-gray-900 dark:text-white", "{meme.inner().name()}" }
                    p { class: "text-sm text-gray-600 dark:text-gray-300", "{meme.inner().description()}" }
                }
            }

            div { class: "flex flex-wrap gap-1 mb-3",
                for tag in meme.inner().tags().iter().take(3) {
                    span {
                        class: "px-2 py-1 bg-gray-100 dark:bg-gray-600 text-xs rounded-full text-gray-700 dark:text-gray-300",
                        "{tag}"
                    }
                }
            }

            div { class: "flex justify-between items-center",
                button {
                    class: "text-blue-500 hover:text-blue-700 text-sm font-medium",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_select.call(meme2.clone());
                    },
                    "View Details"
                }
                button {
                    class: "text-green-500 hover:text-green-700 text-sm font-medium",
                    onclick: move |e| {
                        e.stop_propagation();
                        // Use meme functionality
                    },
                    "Use Meme"
                }
            }
        }
    }
}

#[component]
fn MemeDetailsModal(cx: ScopeState, meme: AnyMemeWrapper, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex justify-between items-start mb-4",
                    div { class: "flex items-center gap-3",
                        span { class: "text-3xl", "{meme.inner().emoji()}" }
                        div {
                            h3 { class: "font-medium text-gray-900 dark:text-white", "{meme.inner().name()}" }
                            p { class: "text-sm text-gray-600 dark:text-gray-300", "{meme.inner().description()}" }
                        }
                    }
                    button {
                        class: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                div { class: "mb-4",
                    h4 { class: "font-medium text-gray-900 dark:text-white mb-2", "Description" }
                    p { class: "text-gray-700 dark:text-gray-300", "{meme.inner().description()}" }
                }

                div { class: "mb-4",
                    h4 { class: "font-medium text-gray-900 dark:text-white mb-2", "Content" }
                    div { class: "bg-gray-100 dark:bg-gray-700 p-3 rounded-lg",
                        pre { class: "text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap", "{meme.inner().content()}" }
                    }
                }

                div { class: "mb-6",
                    h4 { class: "font-medium text-gray-900 dark:text-gray-300 mb-2", "Tags" }
                    div { class: "flex flex-wrap gap-2",
                        for tag in meme.inner().tags().iter() {
                            span {
                                class: "px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-sm rounded-full",
                                "{tag}"
                            }
                        }
                    }
                }

                div { class: "flex gap-3",
                    button {
                        class: "bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600 transition-colors",
                        onclick: move |_| {
                            // Copy to clipboard
                        },
                        "📋 Copy"
                    }
                    button {
                        class: "bg-green-500 text-white px-4 py-2 rounded-lg hover:bg-green-600 transition-colors",
                        onclick: move |_| {
                            // Use meme
                        },
                        "✨ Use Meme"
                    }
                    button {
                        class: "bg-purple-500 text-white px-4 py-2 rounded-lg hover:bg-purple-600 transition-colors",
                        onclick: move |_| {
                            // Edit meme
                        },
                        "✏️ Edit"
                    }
                }
            }
        }
    }
}

// Helper function to get emoji for a category string
fn get_emoji_for_category_string(category_str: &str) -> String {
    match category_str {
        "Component Memes" => "🧩",
        "Workflow Memes" => "⚡",
        "Wikidata Memes" => "📚",
        "Crypto Memes" => "🚀",
        "Lean Memes" => "🎯",
        "Fun Memes" => "🎉",
        _ => "❓",
    }.to_string()
}