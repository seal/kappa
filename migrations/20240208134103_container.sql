-- Add migration script here
create table "container"
(
    container_id uuid primary key default gen_random_uuid(),
    language text   not null,
    port integer not null
);
