-- As a style choice, we prefer to avoid plurals in table names, mainly because it makes queries read better.
--
-- For our user table, quoting the table name is recommended by IntelliJ's tooling because `user` is a keyword,
-- though Postgres seems to handle it fine in most contexts either way.
create table "report"
(
    -- Having the table name as part of the primary key column makes it nicer to write joins, e.g.:
    --
    -- select * from "user"
    -- inner join article using (user_id)
    --
    -- as opposed to `inner join article on article.user_id = user.id`, and makes it easier to keep track of primary
    -- keys as opposed to having all PK columns named "id"
    report_id       uuid primary key                                default uuid_generate_v1mc(),

    -- By applying our custom collation we can simply mark this column as `unique` and Postgres will enforce
    -- case-insensitive uniqueness for us, and lookups over `username` will be case-insensitive by default.
    --
    -- Note that this collation doesn't support the `LIKE`/`ILIKE` operators so if you want to do searches
    -- over `username` you will want a separate index with the default collation:
    --
    -- create index on "user" (username collate "ucs_basic");
    --
    -- select * from "user" where (username collate "ucs_basic") ilike ($1 || '%')
    --
    -- We're not doing that here since the Realworld spec doesn't implement a search function for users.
    title      text collate "case_insensitive" unique not null,

    body         text collate "case_insensitive" unique not null,

    -- The Realworld spec doesn't show `bio` as nullable in the `User` object so we assume it's just empty by default.
    published           bool                                   not null default false
);

-- And applying our `updated_at` trigger is as easy as this.
SELECT trigger_updated_at('"report"');
