mod hello_world {
    tonic::include_proto!("helloworld");
}

use tokio::time::Duration;
use hello_world::{HelloResponse, HelloRequest};
use hello_world::greeter_client::GreeterClient;
use hello_world::greeter_server::{Greeter, GreeterServer};
use tonic::{transport::Server, Request, Response, Status};

async fn run_client(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client =
        GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(
        HelloRequest {
            name: name.into(),
        }
    );

    let response = client.say_hello(request)
        .await?;

    println!("Response={:?}",response.into_inner().message);
    Ok(())

}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::test]
async fn test_grpc_req_resp() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    println!("Greeting Server listening on {}", address);

    tokio::spawn(Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(address));

    tokio::spawn(run_client("Andy Dufresne"));

    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
