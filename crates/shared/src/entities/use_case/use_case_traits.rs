pub trait UseCase<Request, Response> {
    fn execute(&self, request: Request) -> Response;
}

pub trait AddEntityUseCase<Request, Response> {
    fn execute(&self, request: Request) -> Response;
}


