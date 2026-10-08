# Ecommerce API

Run the API from this directory with `cargo run`. The service applies SQL
migrations at startup and exposes its versioned endpoints under `/api/v1`.

## Admin API

Every `/api/v1/admin` route requires a valid bearer token. The admin middleware
checks the user's current role in the database on every request; a deleted user
is unauthenticated and a non-admin is forbidden.

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/api/v1/admin/users` | List users |
| POST | `/api/v1/admin/users` | Create a user with an administrator-selected role |
| GET | `/api/v1/admin/users/{id}` | Get a user |
| PATCH | `/api/v1/admin/users/{id}` | Partially update email, name, password, or role |
| DELETE | `/api/v1/admin/users/{id}` | Delete a user |
| GET | `/api/v1/admin/categories` | List categories |
| POST | `/api/v1/admin/categories` | Create a category; its slug is generated from its name |
| GET | `/api/v1/admin/categories/{id}` | Get a category |
| PATCH | `/api/v1/admin/categories/{id}` | Partially update its name or description |
| DELETE | `/api/v1/admin/categories/{id}` | Delete a category not used by products |

List endpoints accept `page` (starting at 1) and `per_page` (default 20,
maximum 100). Responses contain `items`, `page`, `per_page`, and `total`.
User responses omit password hashes. An administrator cannot delete their own
account or change their own role to `user`.

User creation requires `email`, `password`, `name`, and `role` (`admin` or
`user`). User updates accept any non-empty subset of those fields. Category
creation requires `name` and optionally accepts `description`; updates accept
`name` and/or `description`, with `null` clearing the description.
