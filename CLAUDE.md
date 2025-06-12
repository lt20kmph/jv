# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Local Development

- **Run with auto-reload**: `find src templates -type f | entr -r cargo run`
- **Build for production**: `rustup run nightly cargo build --release`
- **Deploy**: `./deploy.sh` (fetches latest, builds with nightly, restarts systemd service)

### Database

- Uses SQLite database (`db.sqlite3`) with rocket_db_pools/sqlx
- Tables are created automatically on startup via `create_tables()` function
- No separate migration system - schema defined in `src/db/queries.rs`

## Architecture Overview

This is a Rust web application using the Rocket framework for a photo blog/gallery system. Key architectural components:

### Web Framework

- **Rocket 0.5.1** with TLS support and secrets
- **Tera templating** for server-side rendering
- **SQLite** database with sqlx for async queries
- **Image processing** using the `image` crate for thumbnails

### Application Structure

- `src/main.rs` - Application entry point and route mounting
- `src/routes/` - HTTP route handlers (galleries, auth, static assets)
- `src/db/queries.rs` - Database operations and table creation
- `src/models/models.rs` - Data structures and form definitions
- `src/auth/pw_utils.rs` - Password hashing with Argon2
- `templates/` - Tera HTML templates with CSS/JS
- `static/` - Static assets (CSS, icons, favicons)
- `img/` - User-uploaded images with UUID-based filenames

### Key Features

- User authentication with email verification
- Role-based access (Reader/Writer roles)
- Gallery creation and management
- Image upload with automatic thumbnail generation
- Session-based authentication with expiry
- Lightbox image viewing
- HTTPS/TLS support with local certificates

### Database Schema

- `users` - User accounts with email verification
- `sessions` - Session tokens with expiry
- `galleries` - Photo galleries with soft delete
- `original_images` - Original uploaded files
- `modified_images` - Processed images with captions

### Security Features

- Argon2 password hashing with salts
- Session token authentication
- HTTPS/TLS encryption
- File upload size limits (15 MiB)
- Role-based authorization middleware

### Configuration

- `Rocket.toml` - Rocket configuration (TLS, database, limits)
- `Cargo.toml` - Dependencies and project metadata
- Environment uses nightly Rust toolchain for deployment

# Testing

- You do not need to run the server to test the code, I am running the server locally and will provide you with feed back on your changes if you like.

