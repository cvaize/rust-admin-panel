use crate::log_map_err;
use crate::redis_connection::RedisPool;
use actix_web::web::Data;
use r2d2::PooledConnection;
use redis::{Client, Commands, Expiry, FromRedisValue, RedisError, ToRedisArgs};
use strum_macros::{Display, EnumString};


#[derive(Debug, Clone)]
pub struct KeyValueService {
    pool: Data<RedisPool>,
}

impl KeyValueService {
    pub fn new(pool: Data<RedisPool>) -> Self {
        Self { pool }
    }

    pub fn get_connection(&self) -> Result<KeyValueConnection, KeyValueServiceError> {
        let conn: PooledConnection<Client> = self.pool.get_ref().get().map_err(log_map_err!(
            KeyValueServiceError::ConnectFail,
            "KeyValueService::ConnectFail"
        ))?;

        Ok(KeyValueConnection::new(conn))
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get(key)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get_ex(key, seconds)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        self.get_connection()?.get_del(key)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.set(key, value)
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.set_ex(key, value, seconds)
    }

    pub fn expire<K: ToRedisArgs>(&self, key: K, seconds: i64) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.expire(key, seconds)
    }

    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<(), KeyValueServiceError> {
        self.get_connection()?.del(key)
    }
}

pub struct KeyValueConnection {
    conn: PooledConnection<Client>,
}

impl KeyValueConnection {
    pub fn new(conn: PooledConnection<Client>) -> Self {
        Self { conn }
    }

    pub fn get<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get(key).map_err(|e| {
            log::error!("{}", format!("KeyValueService::get - {:}", &e).as_str());
            KeyValueServiceError::GetFail
        })?;
        Ok(value)
    }

    pub fn get_ex<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
        seconds: u64,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get_ex(key, Expiry::EX(seconds)).map_err(|e| {
            log::error!("{}", format!("KeyValueService::get_ex - {:}", &e).as_str());
            KeyValueServiceError::GetExFail
        })?;
        Ok(value)
    }

    pub fn get_del<K: ToRedisArgs, V: FromRedisValue>(
        &mut self,
        key: K,
    ) -> Result<Option<V>, KeyValueServiceError> {
        let value = self.conn.get_del(&key).map_err(|e| {
            log::error!("{}", format!("KeyValueService::get_del - {:}", &e).as_str());
            KeyValueServiceError::GetDelFail
        })?;
        Ok(value)
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), KeyValueServiceError> {
        let result: Result<String, RedisError> = self.conn.set(&key, value);
        if let Err(e) = result {
            log::error!("{}", format!("KeyValueService::set - {:}", &e).as_str());
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'set' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::SetFail);
            }
        }
        Ok(())
    }

    pub fn set_ex<K: ToRedisArgs, V: ToRedisArgs>(
        &mut self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), KeyValueServiceError> {
        let result: Result<String, RedisError> = self.conn.set_ex(&key, value, seconds);
        if let Err(e) = result {
            log::error!("{}", format!("KeyValueService::set_ex - {:}", &e).as_str());
            if e.to_string() == "An error was signalled by the server - ResponseError: wrong number of arguments for 'setex' command" {
                self.del(&key)?;
            } else {
                return Err(KeyValueServiceError::SetExFail);
            }
        }
        Ok(())
    }

    pub fn expire<K: ToRedisArgs>(&mut self, key: K, seconds: i64) -> Result<(), KeyValueServiceError> {
        self.conn.expire(key, seconds).map_err(|e| {
            log::error!("{}", format!("KeyValueService::expire - {:}", &e).as_str());
            KeyValueServiceError::ExpireFail
        })
    }

    pub fn del<K: ToRedisArgs>(&mut self, key: K) -> Result<(), KeyValueServiceError> {
        self.conn.del(key).map_err(|e| {
            log::error!("{}", format!("KeyValueService::del - {:}", &e).as_str());
            KeyValueServiceError::DelFail
        })
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum KeyValueServiceError {
    ConnectFail,
    SetFail,
    SetExFail,
    GetFail,
    GetExFail,
    GetDelFail,
    DelFail,
    ExpireFail,
}
