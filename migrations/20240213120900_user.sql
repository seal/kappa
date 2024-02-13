-- Add migration script here
create table "user"
(
    user_id uuid primary key default gen_random_uuid(),
    username text   not null,
    api_key text   not null
);

