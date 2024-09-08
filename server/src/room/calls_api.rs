use rust_call_client::{
    apis::{
        add_a_track_api,
        configuration::{ApiKey, Configuration},
        new_session_api, renegotiate_web_rtc_session_api,
    },
    models::{
        AppsAppIdSessionsNewPostRequest, AppsAppIdSessionsSessionIdRenegotiatePutRequest,
        SessionDescription, TrackObject, TracksRequest,
    },
};
use tracing::warn;

#[derive(Clone)]
pub struct CallsApi {
    app_id: String,
    config: Configuration,
}

impl CallsApi {
    pub fn new(app_id: String, app_secret: String) -> Self {
        let mut config = Configuration::new();
        config.bearer_access_token = Some(app_secret);
        Self { app_id, config }
    }

    pub async fn new_session(&self) -> Option<String> {
        let resp = new_session_api::apps_app_id_sessions_new_post(&self.config, &self.app_id, None)
            .await
            .map_err(|e| warn!("New session failed {e:?}"))
            .ok()?;
        resp.session_id
    }

    pub async fn add_tracks(
        &self,
        session_id: &str,
        sdp: Option<String>,
        tracks: Vec<(Option<String>, Option<String>)>,
        remote_session_id: Option<String>,
    ) -> Option<String> {
        let tracks_request = TracksRequest {
            session_description: if sdp.is_some() {
                Some(Box::new(SessionDescription {
                    sdp,
                    r#type: Some(rust_call_client::models::session_description::Type::Offer),
                }))
            } else {
                None
            },
            tracks: Some(
                tracks
                    .into_iter()
                    .map(|track| TrackObject {
                        location: Some(if remote_session_id.is_some() {
                            rust_call_client::models::track_object::Location::Remote
                        } else {
                            rust_call_client::models::track_object::Location::Local
                        }),
                        mid: track.0,
                        track_name: track.1,
                        session_id: remote_session_id.clone(),
                    })
                    .collect(),
            ),
        };

        let resp = add_a_track_api::apps_app_id_sessions_session_id_tracks_new_post(
            &self.config,
            &self.app_id,
            session_id,
            Some(tracks_request),
        )
        .await
        .ok()?;
        resp.session_description.and_then(|sd| sd.sdp)
    }

    pub async fn renegotiate(&self, session_id: &str, sdp: String) -> Option<()> {
        let resp =
            renegotiate_web_rtc_session_api::apps_app_id_sessions_session_id_renegotiate_put(
                &self.config,
                &self.app_id,
                session_id,
                Some(AppsAppIdSessionsSessionIdRenegotiatePutRequest {
                    session_description: Some(Box::new(SessionDescription {
                        r#type: Some(rust_call_client::models::session_description::Type::Answer),
                        sdp: Some(sdp),
                    })),
                }),
            )
            .await;
        resp.ok().map(|_| ())
    }
}
