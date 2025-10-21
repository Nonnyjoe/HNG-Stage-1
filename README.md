# Stage 1: Word Processor

This guide will help you test the application endpoints and features. The server is built with Rust and Actix-web. Follow the steps below to ensure a successful test.

---

## Prerequisites

- **Rust toolchain:** Install [Rust](https://www.rust-lang.org/tools/install) if you haven't already.
- **Environment Variables:** You need to create a local `.env` file based on [`.env.example`](.env.example) (see below).

---

## Setup

1. **Clone the Repository & Navigate:**

   ```sh
   git clone https://github.com/Nonnyjoe/HNG-Stage-1
   cd HNG-Stage-1
   ```

2. **Create and Configure the Environment File:**

   - Duplicate [`.env.example`](.env.example) into a new file named `.env`.
   - Fill in the required values:
     - `URL` - URL endpoint to run your server
     - `PORT` – Port on which the server will run.

   Example `.env` file:

   ``` javascript
      URL= 127.0.0.1
      PORT=8080
   ```

3. **Build the Application:**

   ```sh
   cargo build
   ```

## Running the Application

Start the server by running:

```sh
cargo run
```

The server will listen on `127.0.0.1:<PORT>` as specified in the `.env` file.

---

## Testing the Endpoints

This app exposes health/user endpoints and string-processing endpoints.

### 1. Health Check
- **URL:** `http://127.0.0.1:<PORT>/api/v1/healthz`
- **Method:** GET

**cURL:**
```sh
curl -i http://127.0.0.1:8080/api/v1/healthz
```
*Reference: [`check_health`](src/routes/healthz.rs)*

### 2. User Info + Cat Fact
- **URL:** `http://127.0.0.1:<PORT>/api/v1/me`
- **Method:** GET

**cURL:**
```sh
curl -i http://127.0.0.1:8080/api/v1/me
```
*Reference: [`me`](src/routes/me.rs)*

### 3. Strings API

The strings API lets you submit strings, fetch details, filter, and delete.

- Submit a string
  - **URL:** `http://127.0.0.1:<PORT>/api/v1/strings`
  - **Method:** POST
  - **Body:** JSON `{ "value": "your sentence" }`
  - **Success (201) cURL:**
    ```sh
    curl -i -X POST \
      -H 'Content-Type: application/json' \
      -d '{"value":"racecar level kayak"}' \
      http://127.0.0.1:8080/api/v1/strings
    ```
  - Possible errors:
    - 400 if `value` is empty or missing
    - 409 if the string already exists

- Get string details by value
  - **URL:** `http://127.0.0.1:<PORT>/api/v1/strings/{string_value}`
  - **Method:** GET
  - Tip: URL-encode spaces (e.g., `hello%20world`).
  - **cURL:**
    ```sh
    curl -i "http://127.0.0.1:8080/api/v1/strings/hello%20world"
    ```
  - Possible errors:
    - 400 if path value is empty
    - 404 if not found

- Filter stored strings via query params
  - **URL:** `http://127.0.0.1:<PORT>/api/v1/strings`
  - **Method:** GET
  - **Query params (all optional):**
    - `is_palindrome` (bool)
    - `min_length` (usize)
    - `max_length` (usize)
    - `word_count` (u32)
    - `contains_character` (char)
  - **Examples:**
    ```sh
    # Palindromes only
    curl -i "http://127.0.0.1:8080/api/v1/strings?is_palindrome=true"

    # Length between 5 and 10
    curl -i "http://127.0.0.1:8080/api/v1/strings?min_length=5&max_length=10"

    # Single-word palindromes containing letter "a"
    curl -i "http://127.0.0.1:8080/api/v1/strings?is_palindrome=true&word_count=1&contains_character=a"
    ```
  - Possible errors:
    - 404 if no strings match the provided filters

- Experimental: filter via natural language
  - **URL:** `http://127.0.0.1:<PORT>/api/v1/strings/filter-by-natural-language?query=<text>`
  - **Method:** GET
  - Example:
    ```sh
    curl -i "http://127.0.0.1:8080/api/v1/strings/filter-by-natural-language?query=all%20single%20word%20palindromic%20strings"
    ```
  - Note: This endpoint is under development and returns placeholder data for now.

- Delete a string by value
  - **URL:** `http://127.0.0.1:<PORT>/api/v1/strings/{string_value}`
  - **Method:** DELETE
  - **cURL:**
    ```sh
    curl -i -X DELETE "http://127.0.0.1:8080/api/v1/strings/hello%20world"
    ```
  - Possible errors:
    - 400 if path value is empty
    - 404 if not found

---

## Additional Testing Scenarios

- **Check Logs:** Review the printed logs in the terminal to verify that the cat fact URL and random page numbers are logged correctly.

---

## Troubleshooting

- **Server Not Starting:** Verify that all required environment variables are set. Check the logs for error messages.
- **Endpoint Failures:** Use a tool like Postman or cURL to test endpoint responses. Ensure that your firewall or network settings do not block requests.

---

## References

- [Main Application](src/main.rs)
- [Configuration](src/config/config.rs)
- [Health Check Route](src/routes/healthz.rs)
- [String processing Route](src/routes/strings.rs)

Happy testing!
