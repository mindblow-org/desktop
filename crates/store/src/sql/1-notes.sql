create table if not exists notes (
  id text primary key,
  kind text not null,
  title text not null default '',
  body text not null default '',
  items text not null default '[]',
  rank real not null,
  created integer not null,
  updated integer not null
);

create index if not exists notes_rank on notes (rank);
