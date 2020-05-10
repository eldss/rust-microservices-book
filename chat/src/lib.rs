#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

mod models;
mod schema;
use models::{Channel, Id, Membership, Message, User};
use schema::{channels, memberships, messages, users};

use chrono::Utc;
use diesel::{
    insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use failure::{format_err, Error};
use std::env;

pub struct Api {
    conn: PgConnection,
}

impl Api {
    /// Establish DB connection
    pub fn connect() -> Result<Self, Error> {
        let default_url = "postgres://postgres:password@localhost:5432".to_string();
        let database_url = env::var("DATABASE_URL").unwrap_or(default_url);
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    /// Register a new user
    pub fn register_user(&self, email: &str) -> Result<User, Error> {
        insert_into(users::table)
            .values(users::email.eq(email))
            .returning((users::id, users::email))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    /// Create a new channel
    pub fn create_channel(
        &self,
        user_id: Id,
        title: &str,
        is_public: bool,
    ) -> Result<Channel, Error> {
        self.conn.transaction::<_, _, _>(|| {
            // Create the channel
            let channel: Channel = insert_into(channels::table)
                .values((
                    channels::user_id.eq(user_id),
                    channels::title.eq(title),
                    channels::is_public.eq(is_public),
                ))
                .returning((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    channels::created_at,
                    channels::updated_at,
                ))
                .get_result(&self.conn)
                .map_err(Error::from)?;

            // Add the user who created it to the channel
            self.add_member(channel.id, user_id)?;

            Ok(channel)
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
