use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

// https://github.com/tokio-rs/website/blob/master/tutorial-code/channels/src/main.rs
// code example

/// provided by the requester and used by the manager task to send the command resopnse back to the requeser
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

/// multiple different commands are multiplexed over a single channel
#[derive(Debug)]
enum Command{
    Get{
        key:String,
        resp:Responder<Option<Bytes>,
    },
    Set{
        key:String,
        val:Bytes,
        resp:Responder<()>,
    }
}


#[tokio::main]
async fn main(){
    // establish a connection to the server
    // that is not working as the client is shared in a wrong way between the tasks
    // let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    // // spawn two tasks, one gets a key, the other sets a key
    // let t1 = tokio::spawn(async{
    //     client.set("foo", "bar".into()).await;
    // });

    // let t2 = tokio::spawn(async{
    //     client.set("foo", "bar".into()).await;

    // });
    // t1.await.unwrap();
    // t2.await.unwrap();

    let (tx, mut rx) = mpsc::channel::<Command>(32);

    // temporary old version
    // let tx2 = tx.clone();
    // tokio::spawn(async move{
    //     let _ = tx.send("sending from first handle").await;
    // });

    // tokio::spawn(async move{
    //     let _ = tx2.send("sending from second handle").await;
    // });

    // while let Some(message) = rx.recv().await{
    //     println!("GOT = {}", message);
    // }


    let manager = tokio::spawn(async move{
      // establish a connection to the server
      let mut client = client::connect("127.0.0.1:6379").await.unwrap();
      // start receiving messages
      while let Some(cmd)= rx.recv().await{
          use Command::*;
          match cmd{
              Get {key, resp}=>{
                  let res = client.get(&key).await;
                    // ignore errors
                  let _ = resp.send(res);
              }
              Set{key, val, resp}=>{
                    let res = client.set(&key, val).await;
                    // ignore errors
                    let _ = resp.send(res);
              }
          }
      }
    });

    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move{
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "foo".to_string(), resp:resp_tx };
        // send the GET request
        tx.send(cmd).await.unwrap();
        // await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move{
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { key: "foo".to_string(), val: "bar".into(), resp: resp_tx };
        // send the SET request
        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

}

