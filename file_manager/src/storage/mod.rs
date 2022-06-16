
pub mod dbstore{
    use axum::body::Bytes;
    use sqlx::{FromRow, pool::PoolConnection, Postgres};

    #[derive(Debug)]
    pub struct ID{
        pub id:i64,
    }
    impl From<i64> for ID {
        fn from(id: i64) -> Self {Self {id}}
    }
    impl From<ID> for i64 {
        fn from(id: ID) -> Self {id.id }
    }
    #[derive(Debug,FromRow)]
    pub struct PgFile{
        pub id:i64,
        pub name:String,
        pub data:Vec<u8>,
    }
    impl PgFile {
        pub fn new(name:&String,data:&Bytes) ->Self{
            Self { id: 0, name: name.clone(), data: data.to_vec() }
        }
        pub async fn get(id:i64,conn:&mut PoolConnection<Postgres>)->Result<Self,sqlx::Error>{
            let file:PgFile = sqlx::query_as
                (r#" SELECT id,name,data from binary_file where id=$1"#)
                .bind(id)
                .fetch_one(conn).await?;
            Ok(file)
        }
        pub async fn save(&mut self,conn:&mut PoolConnection<Postgres>)->Result<i64,sqlx::Error>{
            self.id=sqlx::query_as!(ID,r#"INSERT INTO binary_file(name,data) VALUES ($1,$2) returning id"#,self.name,self.data)
                .fetch_one(conn).await?.into();
            Ok(self.id)
        }
        
    }
}
