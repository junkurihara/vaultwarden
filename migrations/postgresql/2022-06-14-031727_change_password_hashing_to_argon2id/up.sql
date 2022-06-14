-- Your SQL goes here
ALTER TABLE users
  ADD COLUMN password_memory INTEGER NOT NULL default 0;

ALTER TABLE users
  ADD COLUMN password_parallelism INTEGER NOT NULL default 0;


ALTER TABLE sends
  ADD COLUMN password_mem INTEGER default 0;

ALTER TABLE sends
  ADD COLUMN password_para INTEGER default 0;
