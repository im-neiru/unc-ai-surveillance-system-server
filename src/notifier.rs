use std::collections::{BTreeMap, HashMap};

use actix::{
    dev::{MessageResponse, OneshotSender},
    Addr,
};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::ser::SerializeMap;

use crate::models::UserClaims;

#[derive(Default)]
pub struct Notifier {
    clients: HashMap<uuid::Uuid, Client>,
}

impl Notifier {
    pub async fn add_socket(
        &mut self,
        request: &HttpRequest,
        stream: web::Payload,
    ) -> Result<HttpResponse, actix_web::Error> {
        let claims = UserClaims::from_request(request, &mut Payload::None).await?;

        let actor = Actor {
            claims,
            listen_notification: true,
            listen_activation: true,
        };

        let (addr, response) =
            ws::WsResponseBuilder::new(actor, request, stream).start_with_addr()?;
        self.clients
            .insert(claims.session_id, Client { addr, claims });

        Ok(response)
    }
    pub(crate) fn notify(&self, notification: Notification) {
        for client in self.clients.values() {
            client.addr.do_send(notification.clone());
        }
    }
}

pub(crate) struct Actor {
    pub(super) claims: UserClaims,
    pub(super) listen_notification: bool,
    pub(super) listen_activation: bool,
}

pub(crate) enum Response {
    NotificationSent,
    NotificationSkipped,
    SerializationFailed,
}

#[derive(serde::Deserialize)]
pub(crate) struct ClientRequest {
    event: u32,
    listen: bool,
}

#[derive(actix::Message, Clone)]
#[rtype(result = "Response")]
pub(crate) enum Notification {
    NewViolations(Vec<uuid::Uuid>),
    NewActivation(Vec<ActiveEntry>),
}

#[derive(Clone, serde::Serialize)]
pub(crate) struct ActiveEntry {
    pub(crate) id: uuid::Uuid,
    pub(crate) activity: bool,
}

pub(crate) struct Client {
    pub(super) addr: Addr<Actor>,
    pub(super) claims: UserClaims,
}

impl actix::Actor for Actor {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for Actor {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {
        match item {
            Ok(ws::Message::Binary(cbor)) => {
                if let Ok(request) = serde_cbor::from_slice::<ClientRequest>(&cbor) {
                    match request.event {
                        1 => {
                            self.listen_notification = request.listen;
                        }
                        2 => {
                            self.listen_activation = request.listen;
                        }
                        _ => (),
                    }
                }
            }
            Ok(_) => {}
            Err(_error) => {}
        }
    }
}

impl actix::Handler<Notification> for Actor {
    type Result = Response;

    fn handle(&mut self, notification: Notification, ctx: &mut Self::Context) -> Self::Result {
        match notification {
            Notification::NewViolations(_) => {
                if !self.listen_notification {
                    return Response::NotificationSkipped;
                }
            }
            Notification::NewActivation(_) => {
                if !self.listen_activation {
                    return Response::NotificationSkipped;
                }
            }
        }

        if let Ok(bytes) = serde_cbor::to_vec(&notification) {
            ctx.binary(bytes);

            return Response::NotificationSent;
        }

        Response::SerializationFailed
    }
}

impl serde::ser::Serialize for Notification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Notification::NewViolations(ids) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("event", &1u8)?;
                map.serialize_entry("ids", ids)?;

                map.end()
            }
            Notification::NewActivation(activities) => {
                let mut map = serializer.serialize_map(Some(2))?;

                map.serialize_entry("event", &2u8)?;
                map.serialize_entry("activities", activities)?;

                map.end()
            }
        }
    }
}

impl<M> MessageResponse<Actor, M> for Response
where
    M: actix::Message<Result = Response>,
{
    fn handle(self, _: &mut <Actor as actix::Actor>::Context, _: Option<OneshotSender<M::Result>>) {
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Response::NotificationSent => "Notification Sent",
            Response::SerializationFailed => "Serialization Failed",
            Response::NotificationSkipped => "Notification Skipped",
        })
    }
}
