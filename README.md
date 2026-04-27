# Feature Flags Service

A REST API for managing feature flags with global and per-user overrides.

## How to Run

```bash
git clone https://github.com/pamonteiro84/feature_flags.git
cd feature_flags
make run
```

Server starts on `http://localhost:3000`

## How to Test

To run all unit tests:

```bash
make test
# or
cargo test
```

The tests cover the main business logic and validation for feature flag evaluation and API handlers. All tests are implemented as unit tests within the codebase (look for the `#[cfg(test)]` module).

## Endpoints

| Method | URL                            | Description         |
|--------|--------------------------------|---------------------|
| POST   | /flags                         | Create a flag       |
| GET    | /flags/:key                    | Get a flag          |
| PATCH  | /flags/:key                    | Toggle global state |
| PUT    | /flags/:key/overrides/:user_id | Set user override   |
| GET    | /flags/:key/evaluate?user_id=  | Evaluate for a user |

## Example

```bash
# create
curl -X POST http://localhost:3000/flags \
  -H "Content-Type: application/json" \
  -d '{"key": "new-dashboard", "name": "New Dashboard", "enabled": false}'

# get a flag by key
curl -X GET http://localhost:3000/flags/new-dashboard

# enable for everyone
curl -X PATCH http://localhost:3000/flags/new-dashboard \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# override for one user
curl -X PUT http://localhost:3000/flags/new-dashboard/overrides/user-42 \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# evaluate
curl "http://localhost:3000/flags/new-dashboard/evaluate?user_id=user-42"
# {"flag_key":"new-dashboard","user_id":"user-42","enabled":false,"reason":"user_override"}
```

## Design Decisions

- Keys normalized to lowercase — `New-Dashboard` and `new-dashboard` are the same flag
- Keys only accept letters, numbers, hyphens and underscores
- Evaluate returns 404 if flag does not exist — absence is not the same as disabled
- user_id is treated as an opaque string — user management is outside the scope of this service
- Overrides stored separately from flags — O(1) lookup
- Reason field on evaluate — not required but aids debugging
- 422 on missing required fields — Axum's default deserialization behaviour

