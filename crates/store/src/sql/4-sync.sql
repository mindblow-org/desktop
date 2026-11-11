alter table notes add column clock text not null default '{}';
alter table notes add column seq integer not null default 0;
update notes set seq = rowid;
create index if not exists notes_seq on notes (seq);

create table if not exists gone (
  id text primary key,
  at integer not null,
  seq integer not null
);
create index if not exists gone_seq on gone (seq);

create table if not exists links (
  url text primary key,
  token text not null default '',
  pull integer not null default 0,
  push integer not null default 0
);

create table if not exists meta (
  k text primary key,
  v not null
);
insert into meta (k, v) select 'seq', coalesce(max(seq), 0) from notes;
