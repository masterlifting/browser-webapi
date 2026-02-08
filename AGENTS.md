# Repository Guidelines

## Project Structure

- `src/main.rs` boots the Actix Web server; `src/models.rs` holds shared DTOs and error types.
- `src/web_api/` contains REST API routing and handlers.
- `src/browser/` wraps headless Chrome orchestration and page actions.
- API references live in `openapi.yml` and `postman_collection.json`.
- Docker assets are under `.docker/`.

## Build And Run

- `cargo build` compiles the Rust service.
- `cargo run` starts the API locally (reads `.env`).
- `cargo test` runs unit tests if/when added.
- `docker-compose -f .docker/docker-compose.dev.yml up -d --build` runs the dev stack.
- `docker-compose -f .docker/docker-compose.dev.yml down` stops the dev stack.

## Rust Style (Functional Chain)

- Format with `rustfmt` (`rustfmt.toml`, `tab_spaces = 2`).
- Naming: `snake_case` for functions/vars/modules, `PascalCase` for types.
- Prefer expression-oriented code and “functional chains” over imperative control flow:
- Use combinators for `Result` / `Option` / futures: `map`, `map_err`, `and_then`, `map_ok`, `unwrap_or_else`.
- Lift sync fallible work into the chain with `future::ready(...)` rather than branching early.
- Prefer mapping collections to futures and joining with `future::try_join_all(...)` (or `try_join!`) instead of `for` loops when it improves clarity.
- Keep side-effects at the edges; keep helpers small and single-purpose.
- Error mapping pattern:
- Convert external errors close to the source using `Error::Operation(ErrorInfo { message, code: None })`.
- Messages should include selector/tab_id/url and the underlying error.
- Avoid `mut` where possible; prefer immutability, `iter()`/`into_iter()` pipelines, and small inner functions/closures that capture moved values.

## Testing

- No dedicated Rust test directory is present today.
- Validate endpoints using `postman_collection.json`.
- If using Codex skills, `.codex/skills/api-tests` provides HTTP smoke tests aligned with routes.

## Commits And PRs

- Commit messages: `type: summary` (e.g., `refactor: ...`, `chore: ...`).
- PRs: describe API changes and update `openapi.yml` and the Postman collection when routes change.

## Config And Security

- Local configuration is in `.env`; avoid committing secrets.
- Docker paths and temp dirs are configured in `.docker` and environment variables; keep them in sync with docs.
