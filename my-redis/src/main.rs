use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};


#[tokio::main]
async fn main(){
    // bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop{
        // the second items contains the IP and port of the new connection.
        let (socket, _)= listener.accept().await.unwrap();
        tokio::spawn(async move{
            // green thread https://en.wikipedia.org/wiki/Green_thread
            process(socket).await;
        });
    }
}

async fn process(socket:TcpStream){
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;
    let mut db =HashMap::new();

    // the 'connection' lets us read/write redis **frames** instead of byte streams. The 'Connection' type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap(){

        let response = match Command::from_frame(frame).unwrap(){
            Set(cmd)=>{
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd)=>{
                if let Some(value) =db.get(cmd.key()){
                    // 'Frame::Bulk' expects data to be of type 'Bytes'. 
                    // will be covered later in the tutorial. 
                    // for now '&Vec<u8>' is converted to 'Bytes' using 'into()'
                    Frame::Bulk(value.clone().into())
                }else{
                    Frame::Null
                }
            }
            cmd =>panic!("unimplemented {:?}", cmd),

        };
        connection.write_frame(&response).await.unwrap();
    }


    // first variant
    // let mut connection = Connection::new(socket);
    // if let Some(frame)  = connection.read_frame().await.unwrap(){
    //     println!("GOT: {:?}", frame);
    //     // respond with error 
    //     let response = Frame::Error("unimplemented".to_string());
    //     connection.write_frame(&response).await.unwrap();
    // }

}