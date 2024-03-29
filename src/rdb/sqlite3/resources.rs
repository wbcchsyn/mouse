// Copyright 2021 Shin Yoshida
//
// This file is part of Mouse.
//
// Mouse is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Mouse is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Mouse.  If not, see <https://www.gnu.org/licenses/>.

use super::{Error, Master, Slave, Sqlite3Session, SQLITE_CONSTRAINT_CHECK};
use crate::data_types::{AssetValue, ResourceId};
use std::borrow::Borrow;
use std::collections::HashMap;

/// Make sure to create table "resources".
///
/// This method does nothing if the table is.
pub fn create_table<S>(session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    // Creating table
    {
        const SQL: &'static str = r#"
        CREATE TABLE IF NOT EXISTS resources(
            owner BLOB NOT NULL,
            asset_type BLOB NOT NULL,
            value INTEGER NOT NULL,
            CONSTRAINT resource_id_ PRIMARY KEY(owner, asset_type),
            CONSTRAINT value_ CHECK (value >= 0)
        )"#;

        let mut stmt = session.con.stmt_once(SQL)?;
        stmt.step()?;
    }

    // Creating trigger to cleanup
    {
        const SQL: &'static str = r#"
        CREATE TRIGGER IF NOT EXISTS cleanup_resources
            AFTER UPDATE OF value ON resources
            FOR EACH ROW
            WHEN NEW.value = 0
            BEGIN
                DELETE FROM resources WHERE owner = old.owner AND asset_type = old.asset_type;
            END
        "#;

        let mut stmt = session.con.stmt_once(SQL)?;
        stmt.step()?;
    }

    Ok(())
}

/// Upadtes the asset value in RDB table "resources".
///
/// `balances` is an iterator of ([`ResourceId`] , [`AssetValue`] ) or a reference to it.
///
/// For each balance in `balances` , the value of the [`ResourceId`] is increased by the
/// [`AssetValue`]; i.e. if the [`AssetValue`] is greater than 0, the value is increased
/// (depositted), or if the [`AssetValue`] is less than 0, the value is decreased (withdrawn.)
///
/// # Error
///
/// Errors if any [`AssetValue`] is less than 0.
///
/// [`ResourceId`]: crate::data_types::ResourceId
/// [`AssetValue`]: crate::data_types::AssetValue
pub fn update_balance<I, S, B, R, V>(balances: I, session: &mut S) -> Result<(), Error>
where
    I: Iterator<Item = B> + Clone,
    S: Master,
    B: Borrow<(R, V)>,
    R: Borrow<ResourceId>,
    V: Borrow<AssetValue>,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    // Depositting
    {
        const SQL: &'static str = r#"
        INSERT INTO resources (owner, asset_type, value) VALUES(?1, ?2, ?3)
            ON CONFLICT (owner, asset_type) DO UPDATE set value = value + ?3;
        "#;
        let stmt = session.con.stmt(SQL)?;
        for b in balances.clone() {
            let (resource_id, value) = b.borrow();
            // Skip if the balance is not to deposit.
            if *value.borrow() <= 0 {
                continue;
            }
            stmt.bind_blob(1, resource_id.borrow().owner())?;
            stmt.bind_blob(2, resource_id.borrow().asset_type())?;
            stmt.bind_int(3, *value.borrow())?;
            stmt.step()?;
        }
    }

    // Withdrawing
    {
        // Table constraint prevent from that the value will be less than 0.
        const SQL: &'static str = r#"
        UPDATE resources SET value = value + ?3 WHERE owner = ?1 AND asset_type = ?2;
        "#;
        let stmt = session.con.stmt(SQL)?;
        for b in balances {
            let (resource_id, value) = b.borrow();
            // Skip if the balance is not to withdraw.
            if *value.borrow() >= 0 {
                continue;
            }
            stmt.bind_blob(1, resource_id.borrow().owner())?;
            stmt.bind_blob(2, resource_id.borrow().asset_type())?;
            stmt.bind_int(3, *value.borrow())?;
            stmt.step()?;

            // UPDATE SQL does nothing if no such ResourceId is in the table.
            // Tried to withdraw from not charged ResourceId.
            if stmt.last_changes() == 0 {
                return Err(Error::new(SQLITE_CONSTRAINT_CHECK));
            }
        }
    }

    Ok(())
}

/// Fetches the depositted value of each [`ResourceId`] in `resource_ids` .
///
/// The returned value does not has the [`ResourceId`] as the key if the corresponding value is 0.
pub fn fetch<I, S, R>(
    resource_ids: I,
    session: &mut S,
) -> Result<HashMap<ResourceId, AssetValue>, Error>
where
    I: Iterator<Item = R>,
    S: Slave,
    R: Borrow<ResourceId>,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str = r#"
    SELECT value FROM resources WHERE owner = ?1 AND asset_type = ?2;
    "#;
    let stmt = session.con.stmt(SQL)?;

    let mut ret = match resource_ids.size_hint() {
        (n, None) => HashMap::with_capacity(n),
        (_, Some(n)) => HashMap::with_capacity(n),
    };

    for resource_id in resource_ids {
        let resource_id = resource_id.borrow();
        stmt.bind_blob(1, resource_id.owner())?;
        stmt.bind_blob(2, resource_id.asset_type())?;
        if stmt.step()? {
            let value = stmt.column_int(0).unwrap();
            debug_assert_eq!(true, value > 0);
            ret.insert(*resource_id, value);
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdb::sqlite3::{master, slave, Environment};

    const RESOURCE_COUNT: usize = 10;

    fn empty_table() -> Environment {
        let env = Environment::default();
        {
            let mut session = master(&env);
            create_table(&mut session).unwrap();
        }
        env
    }

    /// The AssetValue of each element equals to the index.
    ///
    /// `assert_eq!(i, balances()[i].1)`
    fn balances() -> Vec<(ResourceId, AssetValue)> {
        let mut ret = Vec::with_capacity(RESOURCE_COUNT);
        let mut owner: [u8; 2] = [0, 0];
        let mut asset_type: [u8; 1] = [0];

        for i in 0..RESOURCE_COUNT {
            owner[0] = i as u8;
            asset_type[0] = i as u8;
            let resource_id = unsafe { ResourceId::new(&owner, &asset_type) };
            ret.push((resource_id, i as AssetValue));
        }

        ret
    }

    #[test]
    fn create_table_() {
        let env = Environment::default();
        let mut session = master(&env);

        assert_eq!(true, create_table(&mut session).is_ok());
        assert_eq!(true, create_table(&mut session).is_ok());
    }

    #[test]
    fn update_balance_() {
        let env = empty_table();
        let mut session = master(&env);

        // Depossiting assets.
        assert_eq!(
            true,
            update_balance(balances().iter().skip(1), &mut session).is_ok()
        );

        // The all balances withdraw the all assets.
        assert_eq!(
            true,
            update_balance(
                balances().iter().skip(1).map(|(k, v)| (k, -v)),
                &mut session
            )
            .is_ok()
        );

        // Deposit again.
        assert_eq!(
            true,
            update_balance(balances().iter().skip(1), &mut session).is_ok()
        );

        // Withdrow from not charged ResourceId.
        {
            assert_eq!(
                false,
                update_balance(
                    balances().iter().take(1).map(|(k, _)| (k, -100)),
                    &mut session
                )
                .is_ok()
            );
        }

        // Withdrow too much from charged ResourceId.
        {
            assert_eq!(
                false,
                update_balance(
                    balances().iter().skip(1).map(|(k, _)| (k, -100)),
                    &mut session
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn fetch_from_empty_table() {
        let env = empty_table();
        let mut session = slave(&env);

        let fetched = fetch(balances().iter().map(|(k, _)| k), &mut session);
        assert_eq!(true, fetched.is_ok());

        let fetched = fetched.unwrap();
        assert_eq!(true, fetched.is_empty());
    }

    #[test]
    fn fetch_from_filled_table() {
        let env = empty_table();
        let mut session = master(&env);

        update_balance(balances().iter(), &mut session).unwrap();

        let fetched = fetch(balances().iter().map(|(k, _)| k), &mut session);
        assert_eq!(true, fetched.is_ok());

        let fetched = fetched.unwrap();
        assert_eq!(balances().len() - 1, fetched.len());
        for balance in balances().iter().skip(1) {
            let (k, v) = balance;
            assert_eq!(*v, fetched[k]);
        }

        update_balance(
            balances().iter().skip(1).map(|(k, _)| (k, -1)),
            &mut session,
        )
        .unwrap();

        let fetched = fetch(balances().iter().map(|(k, _)| k), &mut session);
        assert_eq!(true, fetched.is_ok());

        let fetched = fetched.unwrap();
        assert_eq!(balances().len() - 2, fetched.len());
        for balance in balances().iter().skip(2) {
            let (k, v) = balance;
            assert_eq!(*v - 1, fetched[k]);
        }
    }
}
