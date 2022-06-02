-- Revision: basic schemas
--
-- Add description here

begin;

create schema schema1;
create schema schema2;

create type schema2.marital_status as enum ('single', 'married', 'unknown');

set search_path to schema1, schema2;

create function schema2.married() returns marital_status
  as 'select ''single''::marital_status'
  language sql;

create table schema1.person (
  id int primary key generated always as identity,
  name text not null,
  birthdate date,

  -- Not qualifying schema for 'married' function here helps test
  -- for subtle search_path bugs
  relationship_status marital_status default married()
);

create table schema1.pet (
  id int primary key generated always as identity,
  person_id int not null references person (id),
  name text not null,
  species text
);

commit;
