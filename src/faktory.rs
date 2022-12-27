use std::net::TcpStream;

use faktory::Producer;
use r2d2::ManageConnection;

pub struct FaktoryConnectionManager {
    url: String,
}

impl FaktoryConnectionManager {
    pub fn new(url: String) -> Self {
        FaktoryConnectionManager { url }
    }
}

impl ManageConnection for FaktoryConnectionManager {
    type Connection = Producer<TcpStream>;
    type Error = faktory::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Producer::connect(Some(&self.url))
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // faktoryにチェックする方法がない
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
