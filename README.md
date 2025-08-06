# LonkLink (A link shortener)
*Everyone has to build one at some point right?*

## Setup Locally
1. Create a `.env` file based off the `.env.example`
    - For `ADMIN_PASSWORD_HASH`, run `cargo run hash_admin_password`, enter a password, and copy the output
2. Install the SQLX CLI: `cargo install sqlx-cli`
3. Create the database and run migrations: `sqlx database create && sqlx migrate run`
4. Run the app with `cargo run app`
5. To visit the dashboard, go to `/admin/dashboard/`