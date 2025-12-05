use solfunmeme_loader::{AnyMeme, MemeSource, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;

// The concrete Meme struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: MemeCategory,
    pub emoji: String,
    pub content: String,
    pub tags: Vec<String>,
}

// Implement the AnyMeme trait for our concrete Meme.
impl AnyMeme for Meme {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn category_name(&self) -> String {
        category_name(&self.category).to_string() // Use existing helper, return String
    }
    fn category_emoji(&self) -> String {
        category_emoji(&self.category).to_string() // Use existing helper, return String
    }
    fn emoji(&self) -> String {
        self.emoji.clone() // Return String
    }
    fn content(&self) -> String {
        self.content.clone() // Return String
    }
    fn tags(&self) -> &[String] {
        &self.tags
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    // Implement box_clone
    fn box_clone(&self) -> Box<dyn AnyMeme> {
        Box::new(self.clone())
    }
    // Implement equals
    fn equals(&self, other: &dyn AnyMeme) -> bool {
        if let Some(other_meme) = other.as_any().downcast_ref::<Meme>() {
            self == other_meme
        } else {
            false
        }
    }
}

// Existing MemeCategory enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemeCategory {
    ComponentMemes,
    WorkflowMemes,
    WikidataMemes,
    CryptoMemes,
    LeanMemes,
    FunMemes,
}

// Helper functions (existing).
pub fn category_name(category: &MemeCategory) -> &'static str {
    match category {
        MemeCategory::ComponentMemes => "Component Memes",
        MemeCategory::WorkflowMemes => "Workflow Memes",
        MemeCategory::WikidataMemes => "Wikidata Memes",
        MemeCategory::CryptoMemes => "Crypto Memes",
        MemeCategory::LeanMemes => "Lean Memes",
        MemeCategory::FunMemes => "Fun Memes",
    }
}

pub fn category_emoji(category: &MemeCategory) -> &'static str {
    match category {
        MemeCategory::ComponentMemes => "🧩",
        MemeCategory::WorkflowMemes => "⚡",
        MemeCategory::WikidataMemes => "📚",
        MemeCategory::CryptoMemes => "🚀",
        MemeCategory::LeanMemes => "🎯",
        MemeCategory::FunMemes => "🎉",
    }
}

pub fn filter_memes(memes: &[Meme], category: &MemeCategory, search_query: &str) -> Vec<Meme> {
    memes
        .iter()
        .filter(|meme| meme.category == *category)
        .filter(|meme| {
            if search_query.is_empty() {
                true
            } else {
                let query = search_query.to_lowercase();
                meme.name.to_lowercase().contains(&query)
                    || meme.description.to_lowercase().contains(&query)
                    || meme
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query))
            }
        })
        .cloned()
        .collect()
}

// StaticMemeSource implementation of MemeSource.
pub struct StaticMemeSource;

impl MemeSource for StaticMemeSource {
    fn get_all_memes(&self) -> Result<Vec<Box<dyn AnyMeme>>> {
        Ok(get_memes().into_iter().map(|m| m.box_clone()).collect()) // Use box_clone
    }

    fn get_memes_by_category(&self, category: &str) -> Result<Vec<Box<dyn AnyMeme>>> {
        let all_memes = get_memes();
        let filtered_memes: Vec<Box<dyn AnyMeme>> = all_memes
            .into_iter()
            .filter(|m| category_name(&m.category) == category)
            .map(|m| m.box_clone()) // Use box_clone
            .collect();
        Ok(filtered_memes)
    }
}


// Existing get_memes() function.
pub fn get_memes() -> Vec<Meme> {
    vec![
        // Component Memes
        Meme {
            id: String::from("comp_001"),
            name: String::from("Button Bonanza"),
            description: String::from("A collection of animated button components"),
            category: MemeCategory::ComponentMemes,
            emoji: String::from("🎭"),
            content: String::from("rsx! { button { class: \"animate-bounce\", \"Click me!\" } }"),
            tags: vec![
                String::from("button"),
                String::from("animation"),
                String::from("interactive"),
            ],
        },
        Meme {
            id: String::from("comp_002"),
            name: String::from("Card Carousel"),
            description: String::from("Rotating card components with smooth transitions"),
            category: MemeCategory::ComponentMemes,
            emoji: String::from("🎠"),
            content: String::from("rsx! { div { class: \"transform rotate-3d\", \"Card content\" } }"),
            tags: vec![ 
                String::from("card"),
                String::from("carousel"),
                String::from("rotation"),
            ],
        },
        // Workflow Memes
        Meme {
            id: String::from("work_001"),
            name: String::from("State Machine Meme"),
            description: String::from("Visual representation of state transitions"),
            category: MemeCategory::WorkflowMemes,
            emoji: String::from("⚡"),
            content: String::from("State: Loading -> Success -> Error -> Retry"),
            tags: vec![
                String::from("state"),
                String::from("workflow"),
                String::from("transitions"),
            ],
        },
        Meme {
            id: String::from("work_002"),
            name: String::from("Pipeline Flow"),
            description: String::from("Data processing pipeline visualization"),
            category: MemeCategory::WorkflowMemes,
            emoji: String::from("🔄"),
            content: String::from("Input -> Process -> Transform -> Output"),
            tags: vec![
                String::from("pipeline"),
                String::from("data"),
                String::from("processing"),
            ],
        },
        // Wikidata Memes
        Meme {
            id: String::from("wiki_001"),
            name: String::from("Knowledge Graph"),
            description: String::from("Connected knowledge representation"),
            category: MemeCategory::WikidataMemes,
            emoji: String::from("🕸️"),
            content: String::from("Entity -> Property -> Value -> Reference"),
            tags: vec![
                String::from("knowledge"),
                String::from("graph"),
                String::from("entities"),
            ],
        },
        Meme {
            id: String::from("wiki_002"),
            name: String::from("Semantic Web"),
            description: String::from("Linked data relationships"),
            category: MemeCategory::WikidataMemes,
            emoji: String::from("🌐"),
            content: String::from("Subject -> Predicate -> Object"),
            tags: vec![
                String::from("semantic"),
                String::from("linked-data"),
                String::from("rdf"),
            ],
        },
        // Crypto Memes
        Meme {
            id: String::from("crypto_001"),
            name: String::from("To The Moon"),
            description: String::from("Classic crypto enthusiasm meme"),
            category: MemeCategory::CryptoMemes,
            emoji: String::from("🚀"),
            content: String::from("SOL 🚀🌙 HODL 💎🙌"),
            tags: vec![
                String::from("moon"),
                String::from("hodl"),
                String::from("solana"),
            ],
        },
        Meme {
            id: String::from("crypto_002"),
            name: String::from("Diamond Hands"),
            description: String::from("Never selling, always holding"),
            category: MemeCategory::CryptoMemes,
            emoji: String::from("💎"),
            content: String::from("💎🙌 NEVER SELLING 💎🙌"),
            tags: vec![
                String::from("diamond"),
                String::from("hands"),
                String::from("holding"),
            ],
        },
        // Lean Memes
        Meme {
            id: String::from("lean_001"),
            name: String::from("Proof by Contradiction"),
            description: String::from("When the proof doesn't work out"),
            category: MemeCategory::LeanMemes,
            emoji: String::from("🤔"),
            content: String::from("assume ¬P → ⊥ → P (but at what cost?)"),
            tags: vec![
                String::from("proof"),
                String::from("contradiction"),
                String::from("logic"),
            ],
        },
        Meme {
            id: String::from("lean_002"),
            name: String::from("Tactic Soup"),
            description: String::from("When you throw every tactic at the goal"),
            category: MemeCategory::LeanMemes,
            emoji: String::from("🍲"),
            content: String::from("simp; ring; omega; tauto; sorry"),
            tags: vec![
                String::from("tactics"),
                String::from("automation"),
                String::from("sorry"),
            ],
        },
        // Fun Memes
        Meme {
            id: String::from("fun_001"),
            name: String::from("This is Fine"),
            description: String::from("Everything is totally under control"),
            category: MemeCategory::FunMemes,
            emoji: String::from("🔥"),
            content: String::from("🐕☕ \"This is fine\" 🔥🔥🔥"),
            tags: vec![
                String::from("fine"),
                String::from("chaos"),
                String::from("coffee"),
            ],
        },
        Meme {
            id: String::from("fun_002"),
            name: String::from("Distracted Boyfriend"),
            description: String::from("When new tech catches your eye"),
            category: MemeCategory::FunMemes,
            emoji: String::from("👀"),
            content: String::from("Old Framework 😠 Me 👨 New Shiny Framework 😍"),
            tags: vec![
                String::from("distracted"),
                String::from("technology"),
                String::from("frameworks"),
            ],
        },
    ]
}
