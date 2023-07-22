use super::schema::items;
use diesel::dsl::{delete, insert_into};
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::items)]
pub struct Item {
    pub id: i64,
    pub deleted: Option<bool>,
    pub type_: Option<String>,
    pub by: Option<String>,
    pub time: Option<i64>,
    pub text: Option<String>,
    pub dead: Option<bool>,
    pub parent: Option<i64>,
    pub poll: Option<i64>,
    pub url: Option<String>,
    pub score: Option<i64>,
    pub title: Option<String>,
    pub parts: Option<String>,
    pub descendants: Option<i64>,
}

impl Item {
    pub fn create(conn: &mut PgConnection, new_item: &Item) -> QueryResult<usize> {
        insert_into(items::table).values(new_item).execute(conn)
    }

    pub fn delete(conn: &mut PgConnection, item_id: i64) -> QueryResult<usize> {
        delete(items::table.filter(items::id.eq(item_id))).execute(conn)
    }

    // Add additional methods for querying the items table...
}
