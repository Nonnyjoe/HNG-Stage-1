use crate::AppState;
use crate::config::config::{AnalysisResult, TempDatabase};
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use serde_json::Value;
use sha2::{Sha256, Digest};

#[derive(serde::Deserialize)]
struct UserInput {
    value: String,
}

#[derive(serde::Deserialize)]
enum ProcessStringError {
    EmptyInput,
    NotFound,
    Found(AnalysisResult),
}

#[derive(serde::Deserialize, Debug, Clone)]
enum SearchFilter {
    IsPalindrome(bool),
    MinLength(usize),
    MaxLength(usize),
    WordCount(u32),
    ContainsCharacter(char),
}

#[derive(serde::Deserialize, Debug)]
struct StringQuery {
    is_palindrome: Option<bool>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    word_count: Option<u32>,
    contains_character: Option<char>,
}

#[derive(serde::Deserialize, Debug)]
struct QueryParams {
    query: String,
}


#[post("/strings")]
async fn process_string(_data: web::Data<AppState>, input: web::Json<UserInput>) -> impl Responder {

    println!("Received input: {}", input.value);

    match pre_analysis_check(&input.value, &_data.env.db) {
        ProcessStringError::EmptyInput => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": " Invalid request body or missing \"value\" field"
            });
            return HttpResponse::BadRequest().json(json_response);
        },
        ProcessStringError::Found(_result) => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "String already exists in the system",
            });
            return HttpResponse::Conflict().json(json_response);
        },
        ProcessStringError::NotFound => {
            let analysis_result: AnalysisResult = analyse_string(input.value.clone());

            {
                let mut db = _data.env.db.lock().expect("db mutex poisoned");
                db.processed_strings_hash.push(analysis_result.sha256_hash.clone());
                db.processed_results.push(analysis_result.clone());
            }

            return successful_post_string_response(&analysis_result);
        }
    }
}

#[get("/strings/{string_value}")]
async fn get_string_details(_data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let input_value: String = path.into_inner();
    println!("Received input for details: {}", input_value);

    match pre_analysis_check(&input_value, &_data.env.db) {
        ProcessStringError::EmptyInput => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "Input string is empty. Please provide a valid string."
            });
            return HttpResponse::BadRequest().json(json_response);
        },
        ProcessStringError::NotFound => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "String does not exist in the system",
            });
            return HttpResponse::NotFound().json(json_response);
        },
        ProcessStringError::Found(result) => {
            let properties = struct_to_json(&result);
            let data = serde_json::json!({
                "id": result.sha256_hash,
                "value": result.word,
                "properties": properties,
                "created_at": result.created_at
            });
            return HttpResponse::Ok().json(data);
        }
    }
}


#[get("/strings")]
async fn get_strings_filtered(_data: web::Data<AppState>, query: web::Query<StringQuery>) -> impl Responder {
    let q = query.into_inner();
    println!("Received query for filtering: {:?}", q);

    let selected_filters = extract_filters_from_query(&q);

    let filtered_results = apply_filters(_data, selected_filters.clone());

    return process_filter_response(filtered_results, selected_filters);
}

#[get("/strings/filter-by-natural-language")]
async fn filter_by_natural_language(_data: web::Data<AppState>, query: web::Query<QueryParams>) -> impl Responder {
    let q = query.into_inner().query;
    println!("Received natural language query: {}", q);

    let _parsed_filters: serde_json::Map<String, Value> = match first_stage_process(&q) {
        Ok(filters) => filters,
        Err(_e) => match parse_natural_language_query(&q) {
            Ok(filters) => filters,
            Err(e) => {
                let json_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Could not parse the natural language query: {}", e),
                });
                return HttpResponse::BadRequest().json(json_response);
            }
        },
    };

    match filter_database_res_based_on_query(_data, _parsed_filters.clone()) {
        Some(results) => {

            let mut data_array: Vec<serde_json::Value> = Vec::new();

            for result in results.iter() {
                let properties = struct_to_json(result);
                let data = serde_json::json!({
                    "id": result.sha256_hash,
                    "value": result.word,
                    "properties": properties,
                    "created_at": result.created_at,

                });
                data_array.push(data);
            }

            let response = serde_json::json!({
                "data": data_array,
                "count": data_array.len(),
                "interpreted_query": serde_json::json!({
                    "original": q,
                    "parsed_filters": _parsed_filters
                })
            });

            return HttpResponse::Ok().json(response);
        },
        None => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "Unable to parse natural language query",
            });
            return HttpResponse::BadRequest().json(json_response);
        }
    }
}

fn filter_database_res_based_on_query(_data: web::Data<AppState>, filters: serde_json::Map<String, Value>) -> Option<Vec<AnalysisResult>> {
    let db = _data.env.db.lock().expect("db mutex poisoned");
    let mut all_results: Vec<AnalysisResult> = db.processed_results.clone();
    let mut applied_filters_count: i32 = 0;

    for filter in filters {
        println!("Single Filter pair: {:?}", filter);

        match filter.0.as_str() {
            "is_palindrome" => {
                if let Some(value) = filter.1.as_bool() {
                    println!("Filtering by is_palindrome: {}", value);
                    all_results = all_results.into_iter().filter(|res| res.is_palindrome == value).collect::<Vec<AnalysisResult>>();
                    applied_filters_count += 1;
                }
            },
            "min_length" => {
                if let Some(value) = filter.1.as_u64() {
                    println!("Filtering by min_length: {}", value);
                    all_results = all_results.into_iter().filter(|res| res.length as u64 >= value).collect::<Vec<AnalysisResult>>();
                    applied_filters_count += 1;
                }
            },
            "max_length" => {
                if let Some(value) = filter.1.as_u64() {
                    println!("Filtering by max_length: {}", value);
                    all_results = all_results.into_iter().filter(|res| res.length as u64 <= value).collect::<Vec<AnalysisResult>>();
                    applied_filters_count += 1;
                }
            },
            "word_count" => {
                if let Some(value) = filter.1.as_u64() {
                    println!("Filtering by word_count: {}", value);
                    all_results = all_results.into_iter().filter(|res| res.word_count as u64 == value).collect::<Vec<AnalysisResult>>();
                    applied_filters_count += 1;
                }
            },
            "contains_character" => {
                if let Some(value) = filter.1.as_str() {
                    println!("Filtering by contains_character: {}", value);
                    all_results = all_results.into_iter().filter(|res| res.word.contains(value)).collect::<Vec<AnalysisResult>>();
                    applied_filters_count += 1;
                }
            },
            _ => {
                println!("Unknown filter key: {}", filter.0);
            }
        }

    }
    if applied_filters_count == 0 {
        return None;
    } else {
        return Some(all_results);
    }
}   

fn first_stage_process(query: &str) -> Result<serde_json::Map<String, Value>, String> {
    let lower = query.to_lowercase();
    let mut filters: serde_json::Map<String, Value> = serde_json::Map::new();

    for char in 'a'..='z' {
        let char_str = char.to_string();
        if lower.contains(&format!("strings that contain the letter {}", char_str)) || lower.contains(&format!("the letter {}", char_str)) || lower.contains(&format!("the vowel {}", char_str)) || lower.contains(&format!("the consonant {}", char_str)) {
            filters.insert("contains_character".into(), serde_json::json!(char_str));
        }
    }

    if lower.contains("all single word palindromic strings") {
        filters.insert("is_palindrome".into(), serde_json::json!(true));
        filters.insert("word_count".into(), serde_json::json!(1));
        return Ok(filters)
    } else if lower.contains("strings longer than 10 characters") {
        filters.insert("min_length".into(), serde_json::json!(11));
        return Ok(filters)
    } else if lower.contains("palindromic strings that contain the first vowel") {
        filters.insert("is_palindrome".into(), serde_json::json!(true));
        filters.insert("contains_character".into(), serde_json::json!("a"));
        return Ok(filters)
    } else if lower.contains("palindromic strings that contain the last vowel") {
        filters.insert("is_palindrome".into(), serde_json::json!(true));
        filters.insert("contains_character".into(), serde_json::json!("u"));
        return Ok(filters)
    } else {
        if filters.is_empty() {
            return Err("No recognizable filters found in the query".into())
        } else {
            return Ok(filters)
        }
    }
}


fn parse_natural_language_query(query: &str) -> Result<serde_json::Map<String, Value>, String> {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut filters = serde_json::Map::new();


    if lower.contains("palindrom") {
        filters.insert("is_palindrome".into(), serde_json::json!(true));
    }


    if lower.contains("longer than") {
        if let Some(num) = extract_number(&words) {
            filters.insert("min_length".into(), serde_json::json!(num + 1));
        }
    } else if lower.contains("shorter than") {
        if let Some(num) = extract_number(&words) {
            filters.insert("max_length".into(), serde_json::json!(num - 1));
        }
    }


    if lower.contains("single word") || lower.contains("one word") {
        filters.insert("word_count".into(), serde_json::json!(1));
    } else if lower.contains("two words") || lower.contains("double word") {
        filters.insert("word_count".into(), serde_json::json!(2));
    }

    if let Some(letter) = extract_letter(&words) {
        filters.insert("contains_character".into(), letter.to_string().into());
    } else if lower.contains("first vowel") {
        filters.insert("contains_character".into(), "a".into());
    } else if lower.contains("last vowel") {
        filters.insert("contains_character".into(), "u".into());
    } else if lower.contains("vowel") {
        filters.insert("contains_character".into(), "a".into()); 
    } else if lower.contains("first consonant") {
        filters.insert("contains_character".into(), "b".into()); 
    }


    Ok(filters)
    
}

fn extract_number(words: &[&str]) -> Option<usize> {
    for w in words {
        if let Ok(num) = w.parse::<usize>() {
            return Some(num);
        }
    }
    None
}

fn extract_letter(words: &[&str]) -> Option<char> {
    let letters: Vec<char> = ('a'..='z').collect();
    for (i, &w) in words.iter().enumerate() {
        if w == "letter" {
            if let Some(next) = words.get(i + 1) {
                let c = next.chars().next().unwrap_or(' ');
                if letters.contains(&c) {
                    return Some(c);
                }
            }
        }
    }
    None
}

fn process_filter_response(results: Vec<AnalysisResult>, filters: Vec<SearchFilter>) -> HttpResponse {
    if results.is_empty() {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "No strings match the provided filters",
        });
        return HttpResponse::NotFound().json(json_response);
    }

    let mut data_array: Vec<serde_json::Value> = Vec::new();

    for result in results.iter() {
        let properties = struct_to_json(result);
        let data = serde_json::json!({
            "id": result.sha256_hash,
            "value": result.word,
            "properties": properties,
            "created_at": result.created_at,

        });
        data_array.push(data);
    }

    let response = serde_json::json!({
        "data": data_array,
        "count": data_array.len(),
        "filters_applied": enum_to_string(filters.clone())
    });

    return HttpResponse::Ok().json(response);
}


fn enum_to_string(filters: Vec<SearchFilter>) -> Value {
    let mut object = serde_json::json!({});

    for filter in filters.iter() {
        match filter {
            SearchFilter::IsPalindrome(value) => {object["is_palindrome"] = serde_json::json!(value)},
            SearchFilter::MinLength(min) => {object["min_length"] = serde_json::json!(min)},
            SearchFilter::MaxLength(max) => {object["max_length"] = serde_json::json!(max)},
            SearchFilter::WordCount(count) => {object["word_count"] = serde_json::json!(count)},
            SearchFilter::ContainsCharacter(c) => {object["contains_character"] = serde_json::json!(c)},
        }
    }

    return object;
}

fn extract_filters_from_query(query: &StringQuery) -> Vec<SearchFilter> {
    let mut filters = Vec::new();

    if let Some(is_palindrome) = query.is_palindrome {
        filters.push(SearchFilter::IsPalindrome(is_palindrome));
    }
    if let Some(min_length) = query.min_length {
        filters.push(SearchFilter::MinLength(min_length));
    }
    if let Some(max_length) = query.max_length {
        filters.push(SearchFilter::MaxLength(max_length));
    }
    if let Some(word_count) = query.word_count {
        filters.push(SearchFilter::WordCount(word_count));
    }
    if let Some(contains_character) = query.contains_character {
        filters.push(SearchFilter::ContainsCharacter(contains_character));
    }

    filters
}

fn apply_filters(_data: web::Data<AppState>, filters: Vec<SearchFilter>) -> Vec<AnalysisResult> {
    let mut filtered_results;
    
    {
        let db = _data.env.db.lock().expect("db mutex poisoned");
        filtered_results = db.processed_results.clone();
    }
    
    

    for filter in filters {
        filtered_results = match filter {
            SearchFilter::IsPalindrome(value) => {
                filtered_results.into_iter().filter(|res| res.is_palindrome == value).collect()
            },
            SearchFilter::MinLength(min) => {
                filtered_results.into_iter().filter(|res| res.length >= min).collect()
            },
            SearchFilter::MaxLength(max) => {
                filtered_results.into_iter().filter(|res| res.length <= max).collect()
            },
            SearchFilter::WordCount(count) => {
                filtered_results.into_iter().filter(|res| res.word_count as u32 == count).collect()
            },
            SearchFilter::ContainsCharacter(c) => {
                filtered_results.into_iter().filter(|res| res.word.contains(c)).collect()
            },
        };
    }

    filtered_results
}


#[delete("/strings/{string_value}")]
async fn delete_string(_data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let input_value: String = path.into_inner();
    println!("Received input for deletion: {}", input_value);

    match pre_analysis_check(&input_value, &_data.env.db) {
        ProcessStringError::EmptyInput => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "Input string is empty. Please provide a valid string."
            });
            return HttpResponse::BadRequest().json(json_response);
        },
        ProcessStringError::NotFound => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": "String does not exist in the system",
            });
            return HttpResponse::NotFound().json(json_response);
        },
        ProcessStringError::Found(_result) => {
            {
                let mut db = _data.env.db.lock().expect("db mutex poisoned");
                let mut index_to_remove: Option<usize> = None;
                for (i, processed_string) in db.processed_strings_hash.iter().enumerate() {
                    if *processed_string == _result.sha256_hash {
                        index_to_remove = Some(i);
                        break;
                    }
                }
                if let Some(index) = index_to_remove {
                    db.processed_strings_hash.remove(index);
                    db.processed_results.remove(index);
                }
            }

            let json_response = serde_json::json!({
                "status": "success",
                "message": "String successfully deleted from the system",
            });
            return HttpResponse::Ok().json(json_response);
        }
    }
}

fn analyse_string(input: String) -> AnalysisResult {
    let length = input.chars().count();
    let is_palindrome = input.chars().eq(input.chars().rev());
    let unique_characters = input.chars().collect::<std::collections::HashSet<_>>().len();
    let word = input.clone();
    let word_count = input.split_whitespace().count();

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let sha256_hash = format!("{:x}", result);

    let mut character_frequency_map = std::collections::HashMap::new();
    for c in input.chars() {
        *character_frequency_map.entry(c).or_insert(0) += 1;
    }

    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    return AnalysisResult::new(
        length,
        is_palindrome,
        unique_characters,
        word,
        word_count,
        sha256_hash,
        character_frequency_map,
        created_at,
    );
}


fn map_to_struct(map: std::collections::HashMap<char, usize>) -> serde_json::Value {
    let mut json_struct: serde_json::Value = serde_json::json!({});
    for (key, value) in map.iter() {
        json_struct[key.to_string()] = serde_json::json!(value);
    };

    return json_struct;

}

fn pre_analysis_check(input: &String, db: &std::sync::Mutex<TempDatabase>) -> ProcessStringError {
    if input.is_empty() {
        return ProcessStringError::EmptyInput;
    }

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let input_hash = format!("{:x}", result);

    {
        let db = db.lock().expect("db mutex poisoned");
        for (i, processed_string) in db.processed_strings_hash.iter().enumerate() {
            if *processed_string == input_hash {
                return ProcessStringError::Found(db.processed_results[i].clone());
            }
        }
    }

    return ProcessStringError::NotFound;
}

fn struct_to_json(result: &AnalysisResult) -> serde_json::Value {
    serde_json::json!({
        "length": result.length,
        "is_palindrome": result.is_palindrome,
        "unique_characters": result.unique_characters,
        "word_count": result.word_count,
        "sha256_hash": result.sha256_hash,
        "character_frequency_map": map_to_struct(result.character_frequency_map.clone()),
    })
}

fn successful_post_string_response(result: &AnalysisResult) -> HttpResponse {
   
   let properties = struct_to_json(result);
   let data = serde_json::json!({
        "id": result.sha256_hash,
        "value": result.word,
        "properties": properties,
        "created_at": result.created_at
   });

    HttpResponse::Created().json(data)

}