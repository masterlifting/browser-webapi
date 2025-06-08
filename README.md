# Browser Automation WebAPI

A robust web API service for browser automation using headless Chrome. This project provides a REST API that allows you to control Chrome programmatically for tasks like taking screenshots, generating PDFs, scraping web content, and automating form submissions.

## Features

- **Comprehensive REST API**: Full control over browser capabilities
- **Functional Programming Style**: Pure functions with immutable data structures inspired by F#
- **JWT Authentication**: Secure API access with role-based token authentication
- **Screenshot Capture**: Take screenshots of entire pages or specific elements
- **PDF Generation**: Convert webpages to PDF documents with various options
- **Content Extraction**: Retrieve HTML content, page metadata, and DOM elements
- **JavaScript Execution**: Run arbitrary JavaScript in the page context
- **Form Automation**: Fill and submit forms programmatically
- **Rate Limiting**: Built-in protection against API abuse
- **Modular Architecture**: Well-organized codebase following Rust best practices
- **Container Ready**: Docker support for easy deployment
- **Configurable**: Environment-based configuration
- **Comprehensive Logging**: Structured logs for debugging and monitoring
- **API Documentation**: Interactive Swagger/OpenAPI interface
- **CI/CD Integration**: GitHub Actions for automated testing and deployment

## Tech Stack

- **Rust** - Fast, safe, and concurrent programming language
- **Functional Programming** - Inspired by F# with pure functions and immutable data
- **Actix Web** - High-performance, powerful web framework
- **Headless Chrome** - Browser automation via Chrome DevTools Protocol
- **JWT Authentication** - Secure token-based API protection
- **Tokio** - Asynchronous runtime for non-blocking operations
- **Serde** - Efficient and flexible serialization/deserialization
- **Tracing** - Structured, contextual logging system
- **Utoipa/Swagger** - Interactive API documentation
- **Docker** - Container platform for consistent deployment

## Functional Programming Approach

This project adopts a functional programming style inspired by F#, with an emphasis on:

1. **Immutable Data Structures**: Using immutable domain models that cannot be modified after creation
2. **Pure Functions**: Functions with no side effects that return the same output for the same input
3. **Function Composition**: Building complex behaviors by composing simple functions
4. **Pattern Matching**: Handling different cases cleanly and safely
5. **Error Handling**: Using Result types to handle errors in a functional way

Example of our functional approach in browser operations:

```rust
// Create an immutable selector
let selector = Selector::new("#login-button");

// Compose operations using async/await
let result = page::load(&url, &browser)
    .await
    .and_then(|tab| {
        page::input::fill(&username_selector, "user123", &tab).await?;
        page::input::fill(&password_selector, "password", &tab).await?;
        page::mouse::click(&submit_selector, WaitFor::Url("dashboard"), &tab).await
    });

// Handle potential errors with pattern matching
match result {
    Ok(_) => println!("Login successful"),
    Err(BrowserError::NotFound(msg)) => println!("Element not found: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

This functional approach makes the code more predictable, easier to test, and less prone to bugs.

## Authentication System

The API is protected by JWT (JSON Web Token) based authentication with role-based access control:

1. **Token Generation**: Users can obtain a JWT token by authenticating with username/password
2. **Role-Based Access**: Different endpoints require different permission levels (admin, user, readonly)
3. **Token Validation**: All protected endpoints validate the JWT token and check appropriate roles
4. **Secure Headers**: Authentication tokens are passed via HTTP Authorization headers

Example of accessing a protected endpoint:

```bash
# First, obtain a token
curl -X POST "http://localhost:8000/api/v1/auth/token" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "secure_password"}'

# Then use the token to access protected endpoints
curl -X POST "http://localhost:8000/api/v1/browser/screenshot" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "full_page": true}'
```

See the `examples/AUTH_EXAMPLES.md` file for more detailed authentication examples.

## Project Structure

```
browser_automation_webapi/
├── src/
│   ├── api/           - API version and documentation
│   │   ├── docs.rs    - OpenAPI/Swagger documentation
│   │   └── mod.rs     - API module definitions
│   ├── auth/          - Authentication and authorization
│   │   └── mod.rs     - JWT token generation and validation
│   ├── browser/       - Chrome browser automation logic
│   │   ├── domain.rs  - Immutable domain models
│   │   ├── page.rs    - Functional browser operations
│   │   └── mod.rs     - Browser client and setup
│   ├── config/        - Configuration management
│   ├── error/         - Error handling and types
│   ├── handlers/      - API endpoint handlers
│   │   ├── auth_handlers.rs - Authentication endpoints
│   │   └── browser_handlers.rs - Browser operation endpoints
│   ├── middleware/    - Custom middleware (rate limiting, etc.)
│   ├── models/        - Data models for requests/responses
│   ├── routes/        - API route definitions with authentication
│   ├── services/      - Business logic layer
│   ├── utils/         - Utility functions
│   ├── lib.rs         - Library exports
├── examples/          - Example code and usage demonstrations
│   ├── AUTH_EXAMPLES.md - Authentication usage examples
│   └── functional_browser.rs - Functional browser automation example
├── tests/             - Integration and unit tests
│   ├── integration_tests.rs - Main integration tests
│   └── auth_tests.rs - Authentication-specific tests
├── .github/workflows/ - CI/CD configuration
│   └── ci.yml         - GitHub Actions workflow
│   └── main.rs        - Application entry point
├── examples/          - Example usages
├── Cargo.toml         - Rust package configuration
├── Dockerfile         - Docker container definition
├── docker-compose.yml - Docker Compose configuration
├── .env               - Environment variables
├── build.ps1          - Windows build script
└── build.sh           - Linux/macOS build script
```

## Getting Started

### Prerequisites

- **Rust** (1.70.0 or later)
- **Chrome** or **Chromium** browser
- (Optional) Docker and Docker Compose for containerized deployment

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/browser-automation-webapi.git
   cd browser-automation-webapi
   ```

2. **Run using the build script**:
   
   On Windows:
   ```powershell
   .\build.ps1 run
   ```
   
   On Linux/macOS:
   ```bash
   chmod +x build.sh
   ./build.sh run
   ```

3. **Or use Docker**:
   ```bash
   docker-compose up
   ```

The API will be available at `http://localhost:8080/api/v1`.

### Testing the API

Try a simple request to capture a screenshot:

```bash
curl -X POST http://localhost:8080/api/v1/screenshot \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.example.com"}' \
  --output screenshot.png
```

## Configuration Options

Configure the application through environment variables or the `.env` file:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Host to bind the server | `127.0.0.1` |
| `SERVER_PORT` | Port to listen on | `8080` |
| `LOG_LEVEL` | Log verbosity (trace, debug, info, warn, error) | `info` |
| `CHROME_PATH` | Custom path to Chrome executable | System default |
| `PAGE_LOAD_TIMEOUT` | Page load timeout in milliseconds | `30000` |
| `NAVIGATION_TIMEOUT` | Navigation timeout in milliseconds | `30000` |
| `API_RATE_LIMIT` | Maximum requests per minute | `100` |
| `CORS_ALLOWED_ORIGINS` | Comma-separated list of allowed origins | `*` |

## API Documentation

For detailed API documentation, see [API_DOCS.md](./API_DOCS.md).

For usage examples and implementation guides, see [GETTING_STARTED.md](./GETTING_STARTED.md).

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

### Running Examples

```bash
cargo run --example basic_usage
```

## Deployment

### Docker Deployment

1. Build and run the Docker container:
   ```bash
   docker-compose up -d
   ```

2. For production, consider creating a custom `docker-compose.prod.yml` with appropriate settings.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
