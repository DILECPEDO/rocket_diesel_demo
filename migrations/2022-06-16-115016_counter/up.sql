CREATE TABLE counters (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  counter INTEGER NOT NULL
)