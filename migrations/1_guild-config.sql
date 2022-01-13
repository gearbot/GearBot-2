create table guild_config
(
    id             bigint      not null primary key,
    encryption_key bytea       not null,
    left_at        timestamptz null,
    config         jsonb       not null,
    version        int         not null generated always as (cast(substring((config ->> 'version') from 2) as int)) stored
);