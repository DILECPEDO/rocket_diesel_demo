pub mod models;

pub mod actions {
    use super::models::{Counter, NewCounter};
    use crate::diesel::ExpressionMethods;
    use crate::diesel::RunQueryDsl;
    use diesel::associations::HasTable;
    use diesel::pg::PgConnection;
    use diesel::OptionalExtension;

    pub fn add(conn: &PgConnection, to_add_counter: NewCounter) -> diesel::QueryResult<Counter> {
        use crate::schema::counters::dsl::*;
        diesel::insert_into(counters)
            .values(&to_add_counter)
            .on_conflict(name)
            .do_update()
            .set(counter.eq(counter + to_add_counter.counter))
            .get_result::<Counter>(conn)
    }

    pub fn subtract(
        conn: &PgConnection,
        to_subtract_counter: NewCounter,
    ) -> diesel::QueryResult<Counter> {
        use crate::schema::counters::dsl::*;
        diesel::insert_into(counters)
            .values(&to_subtract_counter)
            .on_conflict(name)
            .do_update()
            .set(counter.eq(counter + to_subtract_counter.counter))
            .get_result::<Counter>(conn)
    }

    pub fn get_counter_by_name(
        conn: &PgConnection,
        _name: String,
    ) -> Result<Option<Counter>, diesel::result::Error> {
        use crate::diesel::QueryDsl;
        use crate::schema::counters::dsl::*;
        counters::table()
            .filter(name.eq(_name))
            .first::<Counter>(conn)
            .optional()
    }

    pub fn get_all_counters(conn: &PgConnection) -> diesel::QueryResult<Vec<Counter>> {
        use crate::schema::counters::dsl::*;
        counters::table().load(conn)
    }

    pub mod with_sqlx {
        use sqlx::{pool::PoolConnection, Postgres};

        use crate::database::models::Counter;

        // use postgres::{, NoTls};

        pub async fn all(
            conn: &mut PoolConnection<Postgres>,
        ) -> Result<Vec<Counter>, sqlx::error::Error> {
            sqlx::query_as::<_, Counter>("SELECT * FROM counters")
                .fetch_all(conn)
                .await
        }
    }

    pub mod with_postgres_crate {
        use postgres;

        use crate::database::models::{Counter, NewCounter};

        // use postgres::{, NoTls};

        pub fn all(conn: &mut postgres::Client) -> Result<Vec<Counter>, postgres::Error> {
            let counters = conn
                .query("select * from counters", &[])?
                .iter()
                .map(|row| Counter {
                    // input of get is column name or positional argument
                    // id: row.get(0),
                    id: row.get("id"),
                    name: row.get("name"),
                    counter: row.get("counter"),
                })
                .collect();
            Ok(counters)
        }

        pub fn add(
            conn: &mut postgres::Client,
            to_add_counter: NewCounter,
        ) -> Result<Counter, postgres::Error> {
            let row = conn
                .query_one("INSERT INTO counters (name, counter) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET counter = counters.counter + $2 RETURNING *", &[&to_add_counter.name, &to_add_counter.counter ])?;
            Ok(Counter {
                id: row.get("id"),
                name: row.get("name"),
                counter: row.get("counter"),
            })
        }

        pub fn subtract(
            conn: &mut postgres::Client,
            to_add_counter: NewCounter,
        ) -> Result<Counter, postgres::Error> {
            let row = conn
                .query_one("INSERT INTO counters (name,counter) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET counter = counters.counter + $2 RETURNING *", &[ &to_add_counter.name, &to_add_counter.counter ])?;
            Ok(Counter {
                id: row.get("id"),
                name: row.get("name"),
                counter: row.get("counter"),
            })
        }
    }
}
