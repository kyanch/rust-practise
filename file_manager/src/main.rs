mod storage;

use std::{env, sync::Arc, net::SocketAddr};
use dotenv::dotenv;

use axum::{routing::{post, get}, Router, Extension, response::Html, http::HeaderMap, extract::{ContentLengthLimit, Multipart, Path}, body::Bytes};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::storage::dbstore::PgFile;

struct AppState{
    pub db:PgPool,
}
async fn state_init()->Result<Arc<AppState>,sqlx::Error>{
    dotenv().ok();
    let pgurl= env::var("DATABASE_URL")
        .expect("设置DATABASE_URL以连接数据库");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&pgurl).await?;
    Ok(Arc::new(AppState { db: pool }))
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let state = state_init().await.unwrap();
    let app = Router::new()
        .route("/", get(upload_html))
        .route("/upload",post(upload_file_handler))
        .route("/down/:id",get(download_file_handler))
        .layer(Extension(state));
    let addr = SocketAddr::from(([127,0,0,1],3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await.unwrap();
}

async fn upload_html()->Html<&'static str>{
    Html(r#"
          <!doctype html>                                                             
          <html>                                                                      
              <head>                                                                  
              <meta charset="utf-8">                                                  
                  <title>上传文件</title>                                             
              </head>                                                                 
              <body>                                                                  
                  <form action="/upload" method="post" enctype="multipart/form-data">                               
                      <label>                                                         
                          上传文件：                                                  
                          <input type="file" name="axum_rs_file">                     
                      </label>                                                        
                      <button type="submit">上传文件</button>                         
                  </form>                                                             
              </body>                                                                 
          </html>  
         "#)
}


const MAX_UPLOAD_SIZE:u64 = 1024*1024*200;// 200MB 这里注意nginx也会限制上传体大小
async fn upload_file_handler(app_state:Extension<Arc<AppState>>,
     ContentLengthLimit(mut multipart):ContentLengthLimit<Multipart, MAX_UPLOAD_SIZE>)
                                                               ->Result<(HeaderMap,String),String>
{
    if let Some(file)= multipart.next_field().await.unwrap(){
        let filename = file.file_name().unwrap().to_string();
        let data = file.bytes().await.unwrap();
        let mut file=PgFile::new(&filename, &data);
        let mut conn=app_state.db.acquire().await.unwrap();
        file.save(&mut conn)
            .await.map_err(|err|err.to_string())?;
        println!("{},{}",&filename,data.len());
        cn(format!("文件上传成功")).await
    }
    else{
        cn(String::from("无法上传文件")).await
    }
}

async fn download_file_handler(Path(id):Path<i64>,app_state:Extension<Arc<AppState>>)->Result<(HeaderMap,Bytes),String>{
    let mut conn = app_state.db.acquire().await.unwrap();
    let file=PgFile::get(id,&mut conn).await.unwrap();
    let mut header = HeaderMap::new();
    header.insert(
        axum::http::header::CONTENT_DISPOSITION, 
        format!("attachment; filename=\"{}\"",file.name).parse().unwrap()
        );
    header.insert(
        axum::http::header::CONTENT_LENGTH, 
        file.data.len().into());
    Ok((header,file.data.into()))
}

async fn cn(msg:String)->Result<(HeaderMap,String),String>{
    let mut header = HeaderMap::new();
    header.insert(
        axum::http::header::CONTENT_TYPE, 
        "text/plain;charset=utf-8".parse().unwrap()
        );
    Ok((header,msg))
}
