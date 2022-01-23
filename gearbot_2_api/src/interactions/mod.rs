use std::str::from_utf8;
use std::sync::Arc;

use actix_web::web::Bytes;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use serde_json::Error;
use tracing::{error, warn};
use twilight_model::application::interaction::Interaction;

use crate::State;

mod command;

#[post("/interactions")]
pub async fn handle_interactions(body: Bytes, request: HttpRequest) -> impl Responder {
    // state can not be missing unless actix completely broke
    // and interactions can't be validated without it, so crashing here
    // is preferable for visibility over trying to handle it nicely
    let state = request.app_data::<Arc<State>>().unwrap();

    // Check for signature
    if let (Some(signature), Some(timestamp)) = (
        request.headers().get("X-Signature-Ed25519"),
        request.headers().get("X-Signature-Timestamp"),
    ) {
        // this can be invalid if not send by discord, make sure it's valid hex
        if let Ok(decoded_signature) = hex::decode(signature) {
            if state
                .public_key
                .verify(&[timestamp.as_bytes(), &body].concat(), &*decoded_signature)
                .is_ok()
            {
                // validation passed, interaction is send by discord and can safely be processed
                let interaction_result: Result<Interaction, Error> = serde_json::from_slice(&body);

                // lets see what we got
                return match interaction_result {
                    Ok(interaction) => {
                        // trace!("Incoming interaction: {:?}", interaction);
                        match interaction {
                            Interaction::Ping(_) => HttpResponse::Ok().body("{\"type\": 1}"),
                            Interaction::ApplicationCommand(command) => {
                                command::handle_command(command, state.clone()).await
                            }
                            // Interaction::ApplicationCommandAutocomplete(_) => {}
                            // Interaction::MessageComponent(_) => {}
                            _ => {
                                warn!("Unhandled interaction type received! {:?}", interaction);
                                HttpResponse::InternalServerError().body("")
                            }
                        }
                    }
                    // This shouldn't be possible at all since we already established the payload was send by discord themselves
                    // but well, better this to be safe then crash
                    Err(error) => {
                        match from_utf8(&body) {
                            Ok(string_body) => {
                                error!("Corrupt interaction received! {} ({})", error, string_body)
                            }
                            Err(_) => error!("Corrupt interaction received! {} ({:?})", error, body.to_vec()),
                        }
                        HttpResponse::BadRequest().body("")
                    }
                };
            }
        }
    }

    HttpResponse::Unauthorized().body("")
}
