![alt text](image.png)
# Backend Service
## Project Overview
This is a Rust-based backend API service for DevBox built using the Axum web framework.

## What is DevBox?
Devbox (written in Rust) is similar to devcontainer but devbox can be deployed on cloud as well.


## API Endpoints
- `/devbox/create`: Create new container.
- `/devbox/list`: List containers from DB.
- `/devbox/logs/{id}`: Get detailed logs for a container.
- `/devbox/{id}?action=start|stop|delete`: Start/Stop/Delete container.
- `/config/edit`: API routes for configuration management for anonymous user
- `/config/provider/docker|azure|aws/edit`: Endpoint to configure specific provider.
- `/api-docs/openapi.json`: OpenAPI JSON specification for the API
- `/swagger`: Interactive Swagger UI for API exploration

## Running Locally
The backend server runs on `http://127.0.0.1:8000` by default.

```bash
cargo run
```

Ensure you have Rust installed and dependencies resolved via Cargo.

### CORS Policy
The server allows Cross-Origin Resource Sharing (CORS) for origins like `http://127.0.0.1:3000`, typically used for local frontend development.

## Logging
The backend uses the tracing crate for structured logging. Logs are output to the console.
Logs for building containers are also stored in separate file.

## Notes
- Background tasks update container statuses every 5 seconds asynchronously.
- API documentation is accessible via the `/swagger` endpoint for ease of testing and exploration.
