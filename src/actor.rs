use actix_web_actors::ws;

pub(crate) struct Context;

impl actix::Actor for Context {
    type Context = actix::Context<Self>;
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for Context {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Binary(bytes)) => {}
            Ok(_) => {}
            Err(_error) => {}
        }
    }
}
