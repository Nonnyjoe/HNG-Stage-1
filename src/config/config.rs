#[derive(Debug)]
pub struct Config {
    pub url: String,
    pub port: String,
    pub db: std::sync::Mutex<TempDatabase>,
}

#[derive(Debug, Clone)]
pub struct TempDatabase{
    pub processed_strings_hash: Vec<String>,
    pub processed_results: Vec<AnalysisResult>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AnalysisResult {
    pub length: usize,
    pub is_palindrome: bool,
    pub unique_characters: usize,
    pub word: String,
    pub word_count: usize,
    pub sha256_hash: String,
    pub character_frequency_map: std::collections::HashMap<char, usize>,
    pub created_at: String,
}

impl TempDatabase {
    pub fn new() -> Self {
        Self {
            processed_strings_hash: Vec::new(),
            processed_results: Vec::new(),
        }
    }
}

impl AnalysisResult {
    pub fn new(
        length: usize,
        is_palindrome: bool,
        unique_characters: usize,
        word: String,
        word_count: usize,
        sha256_hash: String,
        character_frequency_map: std::collections::HashMap<char, usize>,
        created_at: String,
    ) -> Self {
        Self {
            length,
            is_palindrome,
            unique_characters,
            word,
            word_count,
            sha256_hash,
            character_frequency_map,
            created_at,
        }
    }
}

impl Config {
    pub fn init() -> Config {
        let port = std::env::var("PORT").expect("PORT must be set");
        let url = std::env::var("URL").expect("URL must be set");

        let db = TempDatabase::new();


        Config {
            port,
            url,
            db: std::sync::Mutex::new(db),
        }
    }
}

// With a Mutex around the DB, Config is Send + Sync via auto-impls.
