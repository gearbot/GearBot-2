create table message
(
    id          bigint  not null primary key,
    content     bytea   null,
    author      bigint  not null,
    channel     bigint  not null,
    guild       bigint  not null,
    stickers    jsonb   null,
    type        int    not null,
    attachments int    not null,
    pinned      bool    not null
) partition by range (id);

create table attachments
(
    id          bigint  not null primary key ,
    message_id  bigint  not null references message (id),
    name        varchar not null,
    description varchar null
);

-- table for storing when we last cleaned a specific partition and what their lower bounds are
create table cleanups
(
    partition       int8 primary key not null,
    last_cleaned_on date             not null unique,
    lower_bound     bigint           not null unique
);


do
$$
    declare
        lower    bigint := 0;
        starter  bigint := 0;
        increase bigint := 0;
    begin
        -- build a discord snowflake based on today's date
        starter := ((extract(epoch FROM current_timestamp) * 1000 - 1420070400000)::bigint::bit(64) << 22)::bigint;
        -- go back 43 days (6 weeks plus 1 day, so we always have a buffer one for inserting the first new one while we rotate)
        increase := (86400000::bit(64) << 22)::bigint;
        lower := starter - (increase::bigint * 41);
        for counter in 0..42
            loop
                -- need to execute from string to concatenate the name into it. give it it's bounds
                execute 'create table message_partition_' || counter ||
                        ' partition of message for values from (' || lower || ') to (' || lower + increase || ')';
                -- teach our cleanup table this partition exists
                insert into cleanups (partition, last_cleaned_on, lower_bound)
                values (counter, current_date - 42 + counter, lower);
                -- move the lower bound up a day for the next iteration
                lower := lower + increase;
            end loop;
    end;
$$;

create function cleanup_if_needed() returns trigger
    language plpgsql
as
$$
declare
    partition_var int8   := 0;
    lower         bigint := 0;
BEGIN
    -- try the replace the oldest date with today, due to the unique constraint on the table this will only work if we have not rotated yet today
    if not (select exists(select 1 from cleanups where last_cleaned_on = current_date)) then
        update cleanups
        set last_cleaned_on=current_date
        where last_cleaned_on = (select min(last_cleaned_on) from cleanups limit 1)
        returning partition, lower_bound into partition_var, lower;

        -- replace the oldest partition with a fresh one
        execute 'drop table message_partition_' || partition_var || ' cascade';
        execute 'create table message_partition_' || partition_var ||
                ' partition of message for values from (' || lower || ') to (' ||
                lower + (86400000::bit(64) << 22)::bigint * 43 || ')';
    end if;
    return null;

end;

$$;


create trigger trigger_cleanup
    after insert
    on message
    for statement
execute procedure cleanup_if_needed()