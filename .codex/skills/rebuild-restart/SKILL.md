---
name: rebuild-restart
description: Run the browser-webapi rebuild + restart workflow (cargo check, Docker rebuild with the dev tag, and restart services). Use when updating browser-webapi code and needing to rebuild the Docker image and restart the embassy-access stack or the embassy-access-browser-webapi container.
---

# Browser WebAPI Rebuild + Restart

## Run cargo check

- From `D:\private\browser-webapi`, run `cargo check`.

## Rebuild Docker image

- Run the standard rebuild command:

```powershell
docker build -t browser-webapi:dev -f .docker/Dockerfile .
```

## Restart services

- Default: restart the whole compose stack from `D:\private\infra\vps\embassy-access`:

```powershell
docker compose -p embassy-access down
docker compose -p embassy-access up -d
```

- Optional faster path: restart only the browser-webapi container after rebuild:

```powershell
docker compose -p embassy-access up -d --no-deps --force-recreate browser-webapi
```

## Notes

- Prefer the full stack restart when troubleshooting; container-only restart is faster for routine iteration.
- never remove the following volumes:
    ```yaml
        volumes:
            postgres-volume:
            pgadmin-volume:
            pgadmin-data:
    ```
