# Browser WebAPI
A robust web API service for browser automation using headless Chrome. 

This project provides a REST API to control Chrome programmatically for tasks like screenshots, PDF generation, content extraction, and form automation.

## Features

- **Comprehensive REST API** — Full control over browser capabilities  
- **Functional Programming Style** — Pure functions with immutable data structures inspired by F#  
- **Content Extraction** — Retrieve HTML, metadata, and DOM elements  
- **Form Automation** — Fill and submit forms programmatically  
- **Modular Architecture** — Well-organized codebase following Rust best practices  
- **Container Ready** — Docker support for easy deployment  
- **Configurable** — Environment-based configuration  
- **API Documentation** — Interactive Swagger / OpenAPI interface


## Functional programming approach

This project adopts a functional style inspired by F#, emphasizing:

1. Immutable data structures that cannot be modified after creation  
2. Pure functions that return the same output for the same input and avoid side effects  
3. Function composition to build complex behavior from simple functions  
4. Pattern matching for clear, safe branching  
5. Error handling using Result types

## Quick Start with Docker Compose

For fast testing use Docker Compose to run the application locally:

1. **Ensure Docker and Docker Compose are installed** on your system.

2. **Run the service**:
   ```bash
   docker-compose -f .docker/docker-compose.dev.yml up -d --build
   ```
5. **Stop the service**:
   ```bash
   docker-compose -f .docker/docker-compose.dev.yml down
   ```

## API reference

For full request/response examples and ready-to-run requests, import the included `postman_collection.json` into Postman.

**Postman collection**

You can find the Postman collection in the repository root: [postman_collection.json](./postman_collection.json).
Import it into Postman (File → Import or drag-and-drop).

**Set the collection variables:**
- `base_url` as `http://localhost:8080`
- `tab_id` should be set after opening a new tab using the result from the `/api/v1/tab/open` endpoint.

### Implemented routes

| Method | Endpoint | Description |
|---|---|---|
| **GET** | `/api/v1/health` | Health / status |
| **POST** | `/api/v1/tab/open` | Open a new browser tab |
| **DELETE** | `/api/v1/tabs/{id}/close` | Close a tab |
| **POST** | `/api/v1/tabs/{id}/fill` | Fill selected inputs |
| **POST** | `/api/v1/tabs/{id}/humanize` | Apply human-like behaviors to avoid detection |
| **POST** | `/api/v1/tabs/{id}/element/click` | Click an element |
| **POST** | `/api/v1/tabs/{id}/element/exists` | Check if element exists (returns "true"/"false") |
| **POST** | `/api/v1/tabs/{id}/element/extract` | Extract text content or attribute value from an element |
| **POST** | `/api/v1/tabs/{id}/element/execute` | Execute JavaScript on an element |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.