alter table notes add column images text not null default '[]';

create table if not exists blobs (
  id text primary key,
  mime text not null,
  data blob not null
);
