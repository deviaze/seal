// Code adapted from Lune's TableBuilder: https://github.com/lune-org/lune/blob/main/crates/lune-utils/src/table_builder.rs
// This file is licensed under the Mozilla Public License (MPL 2.0): https://github.com/lune-org/lune/blob/main/LICENSE.txt
// as that's Lune's license

#![allow(clippy::missing_errors_doc)]
#![allow(dead_code)]

use mluau::prelude::*;

/**
    Utility struct for building Lua tables.
*/
pub struct TableBuilder<'luau> {
    luau: &'luau Lua,
    tab: LuaTable,
}

impl<'luau> TableBuilder<'luau> {
    /**
        Creates a new table builder.
    */
    pub fn create(luau: &'luau Lua) -> LuaResult<Self> {
        let tab = luau.create_table()?;
        Ok(Self { luau, tab })
    }

    /**
        Adds a new key-value pair to the table.

        This will overwrite any value that already exists.
    */
    pub fn with_value<K, V>(self, key: K, value: V) -> LuaResult<Self>
    where
        K: IntoLua,
        V: IntoLua,
    {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    /**
        Adds multiple key-value pairs to the table.

        This will overwrite any values that already exist.
    */
    pub fn with_values<K, V>(self, values: Vec<(K, V)>) -> LuaResult<Self>
    where
        K: IntoLua,
        V: IntoLua,
    {
        for (key, value) in values {
            self.tab.raw_set(key, value)?;
        }
        Ok(self)
    }

    /**
        Adds a new key-value pair to the sequential (array) section of the table.

        This will not overwrite any value that already exists,
        instead adding the value to the end of the array.
    */
    pub fn with_sequential_value<V>(self, value: V) -> LuaResult<Self>
    where
        V: IntoLua,
    {
        self.tab.raw_push(value)?;
        Ok(self)
    }

    /**
        Adds multiple values to the sequential (array) section of the table.

        This will not overwrite any values that already exist,
        instead adding the values to the end of the array.
    */
    pub fn with_sequential_values<V>(self, values: Vec<V>) -> LuaResult<Self>
    where
        V: IntoLua,
    {
        for value in values {
            self.tab.raw_push(value)?;
        }
        Ok(self)
    }

    /**
        Adds a new key-value pair to the table, with a function value.

        This will overwrite any value that already exists.
    */
    pub fn with_function<K, A, R, F>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua,
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&Lua, A) -> LuaResult<R> + 'static, //+ MaybeSend + 'static,
    {
        let f = self.luau.create_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    pub fn with_function_mut<K, A, R, F>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua,
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: FnMut(&Lua, A) -> LuaResult<R> + 'static,
    {
        let f = self.luau.create_function_mut(func)?;
        self.with_value(key, LuaValue::Function(f)) 
    }

    /*
        Adds a new key-value pair to the table, with an async function value.

        This will overwrite any value that already exists.
    */
//    pub fn with_async_function<K, A, R, F, FR>(self, key: K, func: F) -> LuaResult<Self>
//    where
//        K: IntoLua,
//        A: FromLuaMulti,
//        R: IntoLuaMulti,
//        F: Fn(Lua, A) -> FR + MaybeSend + 'static,
//        FR: Future<Output = LuaResult<R>> + 'static,
//   {
//        let f = self.luau.create_async_function(func)?;
//       self.with_value(key, LuaValue::Function(f))
//    }

    /**
        Adds a metatable to the table.

        This will overwrite any metatable that already exists.
    */
    pub fn with_metatable(self, table: LuaTable) -> LuaResult<Self> {
        self.tab.set_metatable(Some(table))?;
        Ok(self)
    }

    /**
        Builds the table as a read-only table.

        This will prevent any *direct* modifications to the table.
    */
    pub fn build_readonly(self) -> LuaResult<LuaTable> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }

    /**
        Builds the table.
    */
    pub fn build(self) -> LuaResult<LuaTable> {
        Ok(self.tab)
    }
}
