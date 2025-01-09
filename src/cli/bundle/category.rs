use std::fmt;

const CONFIDENCE_THRESHOLD: f64 = 0.8;

const OSX_APP_CATEGORY_PREFIX: &str = "public.app-category.";

// TODO: Right now, these categories correspond to LSApplicationCategoryType values for OS X. There are also some additional GNOME registered categories that don't fit these; we should add those here too.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppCategory {
    Business,
    DeveloperTool,
    Education,
    Entertainment,
    Finance,
    Game,
    ActionGame,
    AdventureGame,
    ArcadeGame,
    BoardGame,
    CardGame,
    CasinoGame,
    DiceGame,
    EducationalGame,
    FamilyGame,
    KidsGame,
    MusicGame,
    PuzzleGame,
    RacingGame,
    RolePlayingGame,
    SimulationGame,
    SportsGame,
    StrategyGame,
    TriviaGame,
    WordGame,
    GraphicsAndDesign,
    HealthcareAndFitness,
    Lifestyle,
    Medical,
    Music,
    News,
    Photography,
    Productivity,
    Reference,
    SocialNetworking,
    Sports,
    Travel,
    Utility,
    Video,
    Weather,
}

impl AppCategory {
    /// Given a string, returns the `AppCategory` it refers to, or the closest
    /// string that the user might have intended (if any).
    pub fn from_str(input: &str) -> Result<AppCategory, Option<&'static str>> {
        // Canonicalize input:
        let mut input = input.to_ascii_lowercase();
        if input.starts_with(OSX_APP_CATEGORY_PREFIX) {
            input = input.split_at(OSX_APP_CATEGORY_PREFIX.len()).1.to_string();
        }
        input = input.replace(' ', "");
        input = input.replace('-', "");

        // Find best match:
        let mut best_confidence = 0.0;
        let mut best_category: Option<AppCategory> = None;
        for &(string, category) in CATEGORY_STRINGS.iter() {
            if input == string {
                return Ok(category);
            }
            let confidence = strsim::jaro_winkler(&input, string);
            if confidence >= CONFIDENCE_THRESHOLD && confidence > best_confidence {
                best_confidence = confidence;
                best_category = Some(category);
            }
        }
        Err(best_category.map(AppCategory::canonical))
    }

    /// Map an AppCategory to the string we recommend to use in Cargo.toml if
    /// the users misspells the category name.
    fn canonical(self) -> &'static str {
        match self {
            AppCategory::Business => "Business",
            AppCategory::DeveloperTool => "Developer Tool",
            AppCategory::Education => "Education",
            AppCategory::Entertainment => "Entertainment",
            AppCategory::Finance => "Finance",
            AppCategory::Game => "Game",
            AppCategory::ActionGame => "Action Game",
            AppCategory::AdventureGame => "Adventure Game",
            AppCategory::ArcadeGame => "Arcade Game",
            AppCategory::BoardGame => "Board Game",
            AppCategory::CardGame => "Card Game",
            AppCategory::CasinoGame => "Casino Game",
            AppCategory::DiceGame => "Dice Game",
            AppCategory::EducationalGame => "Educational Game",
            AppCategory::FamilyGame => "Family Game",
            AppCategory::KidsGame => "Kids Game",
            AppCategory::MusicGame => "Music Game",
            AppCategory::PuzzleGame => "Puzzle Game",
            AppCategory::RacingGame => "Racing Game",
            AppCategory::RolePlayingGame => "Role-Playing Game",
            AppCategory::SimulationGame => "Simulation Game",
            AppCategory::SportsGame => "Sports Game",
            AppCategory::StrategyGame => "Strategy Game",
            AppCategory::TriviaGame => "Trivia Game",
            AppCategory::WordGame => "Word Game",
            AppCategory::GraphicsAndDesign => "Graphics and Design",
            AppCategory::HealthcareAndFitness => "Healthcare and Fitness",
            AppCategory::Lifestyle => "Lifestyle",
            AppCategory::Medical => "Medical",
            AppCategory::Music => "Music",
            AppCategory::News => "News",
            AppCategory::Photography => "Photography",
            AppCategory::Productivity => "Productivity",
            AppCategory::Reference => "Reference",
            AppCategory::SocialNetworking => "Social Networking",
            AppCategory::Sports => "Sports",
            AppCategory::Travel => "Travel",
            AppCategory::Utility => "Utility",
            AppCategory::Video => "Video",
            AppCategory::Weather => "Weather",
        }
    }

    /// Map an AppCategory to the closest set of GNOME desktop registered
    /// categories that matches that category.
    pub fn gnome_desktop_categories(&self) -> &'static str {
        match &self {
            AppCategory::Business => "Office;",
            AppCategory::DeveloperTool => "Development;",
            AppCategory::Education => "Education;",
            AppCategory::Entertainment => "Network;",
            AppCategory::Finance => "Office;Finance;",
            AppCategory::Game => "Game;",
            AppCategory::ActionGame => "Game;ActionGame;",
            AppCategory::AdventureGame => "Game;AdventureGame;",
            AppCategory::ArcadeGame => "Game;ArcadeGame;",
            AppCategory::BoardGame => "Game;BoardGame;",
            AppCategory::CardGame => "Game;CardGame;",
            AppCategory::CasinoGame => "Game;",
            AppCategory::DiceGame => "Game;",
            AppCategory::EducationalGame => "Game;Education;",
            AppCategory::FamilyGame => "Game;",
            AppCategory::KidsGame => "Game;KidsGame;",
            AppCategory::MusicGame => "Game;",
            AppCategory::PuzzleGame => "Game;LogicGame;",
            AppCategory::RacingGame => "Game;",
            AppCategory::RolePlayingGame => "Game;RolePlaying;",
            AppCategory::SimulationGame => "Game;Simulation;",
            AppCategory::SportsGame => "Game;SportsGame;",
            AppCategory::StrategyGame => "Game;StrategyGame;",
            AppCategory::TriviaGame => "Game;",
            AppCategory::WordGame => "Game;",
            AppCategory::GraphicsAndDesign => "Graphics;",
            AppCategory::HealthcareAndFitness => "Science;",
            AppCategory::Lifestyle => "Education;",
            AppCategory::Medical => "Science;MedicalSoftware;",
            AppCategory::Music => "AudioVideo;Audio;Music;",
            AppCategory::News => "Network;News;",
            AppCategory::Photography => "Graphics;Photography;",
            AppCategory::Productivity => "Office;",
            AppCategory::Reference => "Education;",
            AppCategory::SocialNetworking => "Network;",
            AppCategory::Sports => "Education;Sports;",
            AppCategory::Travel => "Education;",
            AppCategory::Utility => "Utility;",
            AppCategory::Video => "AudioVideo;Video;",
            AppCategory::Weather => "Science;",
        }
    }

    /// Map an AppCategory to the closest LSApplicationCategoryType value that
    /// matches that category.
    pub fn osx_application_category_type(&self) -> &'static str {
        match &self {
            AppCategory::Business => "public.app-category.business",
            AppCategory::DeveloperTool => "public.app-category.developer-tools",
            AppCategory::Education => "public.app-category.education",
            AppCategory::Entertainment => "public.app-category.entertainment",
            AppCategory::Finance => "public.app-category.finance",
            AppCategory::Game => "public.app-category.games",
            AppCategory::ActionGame => "public.app-category.action-games",
            AppCategory::AdventureGame => "public.app-category.adventure-games",
            AppCategory::ArcadeGame => "public.app-category.arcade-games",
            AppCategory::BoardGame => "public.app-category.board-games",
            AppCategory::CardGame => "public.app-category.card-games",
            AppCategory::CasinoGame => "public.app-category.casino-games",
            AppCategory::DiceGame => "public.app-category.dice-games",
            AppCategory::EducationalGame => "public.app-category.educational-games",
            AppCategory::FamilyGame => "public.app-category.family-games",
            AppCategory::KidsGame => "public.app-category.kids-games",
            AppCategory::MusicGame => "public.app-category.music-games",
            AppCategory::PuzzleGame => "public.app-category.puzzle-games",
            AppCategory::RacingGame => "public.app-category.racing-games",
            AppCategory::RolePlayingGame => "public.app-category.role-playing-games",
            AppCategory::SimulationGame => "public.app-category.simulation-games",
            AppCategory::SportsGame => "public.app-category.sports-games",
            AppCategory::StrategyGame => "public.app-category.strategy-games",
            AppCategory::TriviaGame => "public.app-category.trivia-games",
            AppCategory::WordGame => "public.app-category.word-games",
            AppCategory::GraphicsAndDesign => "public.app-category.graphics-design",
            AppCategory::HealthcareAndFitness => "public.app-category.healthcare-fitness",
            AppCategory::Lifestyle => "public.app-category.lifestyle",
            AppCategory::Medical => "public.app-category.medical",
            AppCategory::Music => "public.app-category.music",
            AppCategory::News => "public.app-category.news",
            AppCategory::Photography => "public.app-category.photography",
            AppCategory::Productivity => "public.app-category.productivity",
            AppCategory::Reference => "public.app-category.reference",
            AppCategory::SocialNetworking => "public.app-category.social-networking",
            AppCategory::Sports => "public.app-category.sports",
            AppCategory::Travel => "public.app-category.travel",
            AppCategory::Utility => "public.app-category.utilities",
            AppCategory::Video => "public.app-category.video",
            AppCategory::Weather => "public.app-category.weather",
        }
    }
}

impl<'d> serde::Deserialize<'d> for AppCategory {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<AppCategory, D::Error> {
        deserializer.deserialize_str(AppCategoryVisitor { did_you_mean: None })
    }
}

struct AppCategoryVisitor {
    did_you_mean: Option<&'static str>,
}

impl<'d> serde::de::Visitor<'d> for AppCategoryVisitor {
    type Value = AppCategory;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.did_you_mean {
            Some(string) => {
                write!(
                    formatter,
                    "a valid app category string (did you mean \"{string}\"?)"
                )
            }
            None => write!(formatter, "a valid app category string"),
        }
    }

    fn visit_str<E: serde::de::Error>(mut self, value: &str) -> Result<AppCategory, E> {
        match AppCategory::from_str(value) {
            Ok(category) => Ok(category),
            Err(did_you_mean) => {
                self.did_you_mean = did_you_mean;
                let unexp = serde::de::Unexpected::Str(value);
                Err(serde::de::Error::invalid_value(unexp, &self))
            }
        }
    }
}

const CATEGORY_STRINGS: &[(&str, AppCategory)] = &[
    ("actiongame", AppCategory::ActionGame),
    ("actiongames", AppCategory::ActionGame),
    ("adventuregame", AppCategory::AdventureGame),
    ("adventuregames", AppCategory::AdventureGame),
    ("arcadegame", AppCategory::ArcadeGame),
    ("arcadegames", AppCategory::ArcadeGame),
    ("boardgame", AppCategory::BoardGame),
    ("boardgames", AppCategory::BoardGame),
    ("business", AppCategory::Business),
    ("cardgame", AppCategory::CardGame),
    ("cardgames", AppCategory::CardGame),
    ("casinogame", AppCategory::CasinoGame),
    ("casinogames", AppCategory::CasinoGame),
    ("developer", AppCategory::DeveloperTool),
    ("developertool", AppCategory::DeveloperTool),
    ("developertools", AppCategory::DeveloperTool),
    ("development", AppCategory::DeveloperTool),
    ("dicegame", AppCategory::DiceGame),
    ("dicegames", AppCategory::DiceGame),
    ("education", AppCategory::Education),
    ("educationalgame", AppCategory::EducationalGame),
    ("educationalgames", AppCategory::EducationalGame),
    ("entertainment", AppCategory::Entertainment),
    ("familygame", AppCategory::FamilyGame),
    ("familygames", AppCategory::FamilyGame),
    ("finance", AppCategory::Finance),
    ("fitness", AppCategory::HealthcareAndFitness),
    ("game", AppCategory::Game),
    ("games", AppCategory::Game),
    ("graphicdesign", AppCategory::GraphicsAndDesign),
    ("graphicsanddesign", AppCategory::GraphicsAndDesign),
    ("graphicsdesign", AppCategory::GraphicsAndDesign),
    ("healthcareandfitness", AppCategory::HealthcareAndFitness),
    ("healthcarefitness", AppCategory::HealthcareAndFitness),
    ("kidsgame", AppCategory::KidsGame),
    ("kidsgames", AppCategory::KidsGame),
    ("lifestyle", AppCategory::Lifestyle),
    ("logicgame", AppCategory::PuzzleGame),
    ("medical", AppCategory::Medical),
    ("medicalsoftware", AppCategory::Medical),
    ("music", AppCategory::Music),
    ("musicgame", AppCategory::MusicGame),
    ("musicgames", AppCategory::MusicGame),
    ("news", AppCategory::News),
    ("photography", AppCategory::Photography),
    ("productivity", AppCategory::Productivity),
    ("puzzlegame", AppCategory::PuzzleGame),
    ("puzzlegames", AppCategory::PuzzleGame),
    ("racinggame", AppCategory::RacingGame),
    ("racinggames", AppCategory::RacingGame),
    ("reference", AppCategory::Reference),
    ("roleplaying", AppCategory::RolePlayingGame),
    ("roleplayinggame", AppCategory::RolePlayingGame),
    ("roleplayinggames", AppCategory::RolePlayingGame),
    ("rpg", AppCategory::RolePlayingGame),
    ("simulationgame", AppCategory::SimulationGame),
    ("simulationgames", AppCategory::SimulationGame),
    ("socialnetwork", AppCategory::SocialNetworking),
    ("socialnetworking", AppCategory::SocialNetworking),
    ("sports", AppCategory::Sports),
    ("sportsgame", AppCategory::SportsGame),
    ("sportsgames", AppCategory::SportsGame),
    ("strategygame", AppCategory::StrategyGame),
    ("strategygames", AppCategory::StrategyGame),
    ("travel", AppCategory::Travel),
    ("triviagame", AppCategory::TriviaGame),
    ("triviagames", AppCategory::TriviaGame),
    ("utilities", AppCategory::Utility),
    ("utility", AppCategory::Utility),
    ("video", AppCategory::Video),
    ("weather", AppCategory::Weather),
    ("wordgame", AppCategory::WordGame),
    ("wordgames", AppCategory::WordGame),
];