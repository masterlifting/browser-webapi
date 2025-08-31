# Browser WebAPI
A robust web API service for browser automation using headless Chrome. This project provides a REST API to control Chrome programmatically for tasks like screenshots, PDF generation, content extraction, and form automation.

## Features

- **Comprehensive REST API** — Full control over browser capabilities  
- **Functional Programming Style** — Pure functions with immutable data structures inspired by F#  
- **Content Extraction** — Retrieve HTML, metadata, and DOM elements  
- **Form Automation** — Fill and submit forms programmatically  
- **Modular Architecture** — Well-organized codebase following Rust best practices  
- **Container Ready** — Docker support for easy deployment  
- **Configurable** — Environment-based configuration  
- **API Documentation** — Interactive Swagger / OpenAPI interface

## Tech stack

- Rust — Fast, safe, concurrent language  
- Functional programming — F#-inspired pure functions and immutability  
- Actix Web — High-performance web framework  
- Headless Chrome — Automation via Chrome DevTools Protocol  
- Tokio — Async runtime  
- Serde — Serialization / deserialization  
- Tracing — Structured logging  
- Docker — Consistent deployment

## Functional programming approach

This project adopts a functional style inspired by F#, emphasizing:

1. Immutable data structures that cannot be modified after creation  
2. Pure functions that return the same output for the same input and avoid side effects  
3. Function composition to build complex behavior from simple functions  
4. Pattern matching for clear, safe branching  
5. Error handling using Result types

# Browser WebAPI — API reference

This README provides a short, accurate API summary. For full request/response examples and ready-to-run requests, import the included `postman_collection.json` into Postman.

Postman collection

You can find the Postman collection in the repository root: [postman_collection.json](./postman_collection.json).
Import it into Postman (File → Import or drag-and-drop) and set the collection variables `base_url` and `tab_id` before running requests.

Implemented routes (summary)

| Method | Endpoint | Description |
|---|---|---|
| **GET** | `/api/v1/health` | Health / status |
| **POST** | `/api/v1/tab/open` | Open a new browser tab |
| **GET** | `/api/v1/tabs/{id}/close` | Close a tab |
| **POST** | `/api/v1/tabs/{id}/fill` | Fill selected inputs |
| **POST** | `/api/v1/tabs/{id}/element/click` | Click an element |
| **POST** | `/api/v1/tabs/{id}/element/exists` | Check if element exists (returns "true"/"false") |
| **POST** | `/api/v1/tabs/{id}/element/extract` | Extract text content or attribute value from an element |
| **POST** | `/api/v1/tabs/{id}/element/execute` | Execute JavaScript on an element |