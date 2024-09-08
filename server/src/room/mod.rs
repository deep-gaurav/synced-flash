use axum::{
    extract::{ws::WebSocket, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::{
    message::{Message, UserJoined, UserLeft},
    message_sender::MessageSender,
    params::{HostParams, JoinParams},
    PlayerStatus, RoomProviderError, User, UserMeta, UserState,
};
use leptos::logging::warn;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::AppState;

pub mod calls_api;

#[derive(Error, Debug)]
pub enum RoomJoinError {
    #[error(transparent)]
    RoomProviderError(#[from] RoomProviderError),
}

#[axum::debug_handler]
pub async fn host_room(
    State(app_state): State<AppState>,
    Query(host_params): Query<HostParams>,
    ws: WebSocketUpgrade,
) -> Result<Response, RoomJoinError> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let user_id = Uuid::new_v4();
    let user = User {
        meta: UserMeta {
            id: user_id,
            name: host_params.name,
            state: common::UserState::VideoNotSelected,
        },
        sender: tx,
    };
    let room_id = app_state.rooms.new_room(user).await?;

    Ok(ws.on_upgrade(move |mut msgs| async move {
        msgs.send_message(&Message::ServerMessage(
            common::message::ServerMessage::RoomCreated(room_id.clone()),
        ))
        .await;

        handle_websocket(app_state, &room_id.room_id, user_id, msgs, rx).await;
    }))
}

#[axum::debug_handler]
pub async fn join_room(
    State(app_state): State<AppState>,
    Query(join_params): Query<JoinParams>,
    ws: WebSocketUpgrade,
) -> Result<Response, RoomJoinError> {
    let (tx, rx) = tokio::sync::mpsc::channel(10); // 10 is random here.
    let user_id = Uuid::new_v4();
    let user = User {
        meta: UserMeta {
            id: user_id,
            name: join_params.name,

            state: common::UserState::VideoNotSelected,
        },
        sender: tx,
    };

    let join_info = app_state
        .rooms
        .join_room(&join_params.room_id.to_lowercase(), user)
        .await?;
    let room_id = join_params.room_id;
    if let Some(player_status) = app_state.rooms.get_room_player_status(&room_id).await {
        app_state
            .rooms
            .broadcast_msg_excluding(
                &room_id,
                Message::ServerMessage(common::message::ServerMessage::UserJoined(UserJoined {
                    new_user: join_info.user_id,
                    users: join_info.users.clone(),
                    player_status,
                })),
                &[join_info.user_id],
            )
            .await;
    }
    Ok(ws.on_upgrade(move |mut msgs| async move {
        msgs.send_message(&Message::ServerMessage(
            common::message::ServerMessage::RoomJoined(join_info),
        ))
        .await;

        handle_websocket(app_state, &room_id, user_id, msgs, rx).await;
    }))
}

async fn handle_websocket(
    app_state: AppState,
    room_id: &str,
    user_id: Uuid,
    mut socket: WebSocket,
    mut rx: tokio::sync::mpsc::Receiver<Message>,
) {
    let mut session_id = None;
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => {
                                if handle_message(msg, user_id, room_id, &app_state, &mut session_id, &mut socket).await {
                                    break;
                                }
                            }
                            Err(err) => {
                                warn!("Msg receive error {err:#?}")
                            }
                        }
                    },
                    None => {
                        // User disconnected
                        info!("Received None from socket disconnecting {user_id}");
                        break;
                    },
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        socket.send_message(&msg).await;
                    }
                    None => {
                        // Sender dropped, room closed?
                        info!("Received None from rx disconnecting {user_id}");
                        break;
                    }
                }
            }
        }
    }
    let remaining_users = app_state.rooms.remove_user(room_id, user_id).await;
    info!("Disconnected user {user_id}");
    if let Some(users) = remaining_users {
        if let Some(player_status) = app_state.rooms.get_room_player_status(room_id).await {
            app_state
                .rooms
                .broadcast_msg_excluding(
                    room_id,
                    Message::ServerMessage(common::message::ServerMessage::UserLeft(UserLeft {
                        user_left: user_id,
                        users,
                        player_status,
                    })),
                    &[user_id],
                )
                .await;
        }
    }
}

pub async fn handle_message(
    msg: axum::extract::ws::Message,
    user_id: Uuid,
    room_id: &str,
    app_state: &AppState,
    session_id: &mut Option<String>,
    socker: &mut WebSocket,
) -> bool {
    match msg {
        axum::extract::ws::Message::Text(_) => {
            //ignore
        }
        axum::extract::ws::Message::Binary(data) => {
            let data = bincode::deserialize::<Message>(&data[..]);
            match data {
                Ok(original_message) => {
                    match &original_message {
                        Message::ServerMessage(_) => {
                            //ignore
                        }
                        Message::RTCMessage(message) => match message {
                            common::message::RTCMessage::AddHostSdp(sdp, tracks) => {
                                let new_session_id = app_state.calls_api.new_session(None).await;
                                if let Some((new_session_id, answer)) = new_session_id {
                                    let sdp = app_state
                                        .calls_api
                                        .add_tracks(
                                            &new_session_id,
                                            sdp.clone(),
                                            tracks.clone(),
                                            None,
                                        )
                                        .await;
                                    if let Some(sdp) = sdp {
                                        *session_id = Some(new_session_id.clone());
                                        app_state
                                            .rooms
                                            .with_room_mut(room_id, |room| {
                                                room.tracks = Some((new_session_id, tracks.clone()))
                                            })
                                            .await;
                                        socker
                                            .send_message(&Message::RTCMessage(
                                                common::message::RTCMessage::AddHostRemoteSdp(sdp),
                                            ))
                                            .await;
                                    } else {
                                        warn!("Failed to add tracks")
                                    }
                                } else {
                                    warn!("New session failed")
                                }
                                // socker.send_message(&Message::RTCMessage(
                                //     common::message::RTCMessage::AddHostRemoteSdp((), ())
                                // ))
                            }
                            common::message::RTCMessage::AddHostRemoteSdp(_) => todo!(),

                            common::message::RTCMessage::MakeJoinOffer(sdp) => {
                                if session_id.is_some() {
                                    warn!("Already joined");
                                    return false;
                                }
                                info!("making join offer");
                                if let Some((new_session_id, answer)) =
                                    app_state.calls_api.new_session(Some(sdp.clone())).await
                                {
                                    *session_id = Some(new_session_id);
                                    if let Some(answer) = answer {
                                        socker
                                            .send_message(&Message::RTCMessage(
                                                common::message::RTCMessage::JoinAnswer(answer),
                                            ))
                                            .await;
                                    }
                                }
                            }
                            common::message::RTCMessage::JoinAnswer(_) => todo!(),
                            common::message::RTCMessage::RequestJoinSdp => {
                                if let Some((remote_session, tracks)) = app_state
                                    .rooms
                                    .with_room(room_id, |r| r.tracks.clone())
                                    .await
                                    .flatten()
                                {
                                    let new_session_id = session_id.clone().or(app_state
                                        .calls_api
                                        .new_session(None)
                                        .await
                                        .map(|p| p.0));
                                    if let Some(new_session_id) = new_session_id {
                                        let sdp = app_state
                                            .calls_api
                                            .add_tracks(
                                                &new_session_id,
                                                None,
                                                tracks.clone(),
                                                Some(remote_session),
                                            )
                                            .await;
                                        if let Some(sdp) = sdp {
                                            *session_id = Some(new_session_id);
                                            socker
                                                .send_message(&Message::RTCMessage(
                                                    common::message::RTCMessage::JoinRemoteSdp(sdp),
                                                ))
                                                .await;
                                        } else {
                                            warn!("Failed to add tracks")
                                        }
                                    } else {
                                        warn!("New session failed")
                                    }
                                }
                            }
                            common::message::RTCMessage::JoinRemoteSdp(_) => {
                                warn!("Not expected JoinRemoteSdp on server");
                            }

                            common::message::RTCMessage::DataChannelCreated(_) => {
                                warn!("Not expected DataChannelCreated on server");
                            }
                            common::message::RTCMessage::SendJoinLocalSdp(sdp) => {
                                if let Some(session_id) = &session_id {
                                    match app_state
                                        .calls_api
                                        .renegotiate(session_id, sdp.clone())
                                        .await
                                    {
                                        Some(_) => {
                                            info!("Renegotiation success")
                                        }
                                        None => {
                                            warn!("Renegotiation failed")
                                        }
                                    }
                                }
                            }
                            common::message::RTCMessage::RequestDataChannel(name) => {
                                if let Some(session_id) = &session_id {
                                    if let Some(host_session_id) = app_state
                                        .rooms
                                        .with_room(room_id, |r| {
                                            r.tracks.as_ref().map(|r| r.0.clone())
                                        })
                                        .await
                                        .flatten()
                                    {
                                        if session_id == &host_session_id {
                                            warn!("Host cant request data channel");
                                            return false;
                                        }
                                        let data_channel_sender = app_state
                                            .calls_api
                                            .new_data_channel(session_id, None, name.clone())
                                            .await;
                                        let data_channel_host_sub = app_state
                                            .calls_api
                                            .new_data_channel(
                                                &host_session_id,
                                                Some(session_id.clone()),
                                                name.clone(),
                                            )
                                            .await;
                                        if let (Some(data_channel), Some(data_channel_sub)) =
                                            (data_channel_sender, data_channel_host_sub)
                                        {
                                            socker
                                                .send_message(&Message::RTCMessage(
                                                    common::message::RTCMessage::DataChannelCreated(
                                                        (name.clone(), data_channel),
                                                    ),
                                                ))
                                                .await;
                                            if let Some(host_user_id) = app_state
                                                .rooms
                                                .with_room(room_id, |room| {
                                                    room.users.first().map(|u| u.meta.id)
                                                })
                                                .await
                                                .flatten()
                                            {
                                                app_state.rooms.send_msg_for_user(room_id, host_user_id, Message::RTCMessage(
                                                    common::message::RTCMessage::DataChannelCreated(
                                                        (format!("{user_id}-sub"), data_channel_sub),
                                                    ),
                                                )).await;
                                            }
                                        } else {
                                            warn!("Data channel creation failed")
                                        }
                                    }
                                }
                            }
                        },
                        Message::ClientMessage((sender_id, message)) => {
                            if sender_id == &user_id {
                                match message {
                                    common::message::ClientMessage::Chat(_) => {
                                        app_state
                                            .rooms
                                            .broadcast_msg_excluding(
                                                room_id,
                                                original_message,
                                                &[user_id],
                                            )
                                            .await;
                                    }
                                    common::message::ClientMessage::SelectedVideo(video_name) => {
                                        app_state
                                            .rooms
                                            .with_room_mut(room_id, |room| {
                                                if let Some(user) = room
                                                    .users
                                                    .iter_mut()
                                                    .find(|u| u.meta.id == user_id)
                                                {
                                                    user.meta.state = UserState::VideoSelected(
                                                        video_name.clone(),
                                                    );
                                                }
                                            })
                                            .await;
                                        app_state
                                            .rooms
                                            .broadcast_msg_excluding(
                                                room_id,
                                                original_message,
                                                &[user_id],
                                            )
                                            .await;
                                    }
                                    common::message::ClientMessage::Play(val) => {
                                        app_state
                                            .rooms
                                            .with_room_mut(room_id, |room| {
                                                room.player_status = PlayerStatus::Playing(*val);
                                            })
                                            .await;
                                        app_state
                                            .rooms
                                            .broadcast_msg_excluding(
                                                room_id,
                                                original_message,
                                                &[user_id],
                                            )
                                            .await;
                                    }
                                    common::message::ClientMessage::Pause(val) => {
                                        app_state
                                            .rooms
                                            .with_room_mut(room_id, |room| {
                                                room.player_status = PlayerStatus::Paused(*val);
                                            })
                                            .await;
                                        app_state
                                            .rooms
                                            .broadcast_msg_excluding(
                                                room_id,
                                                original_message,
                                                &[user_id],
                                            )
                                            .await;
                                    }
                                    common::message::ClientMessage::Seek(val)
                                    | common::message::ClientMessage::Update(val) => {
                                        app_state
                                            .rooms
                                            .with_room_mut(room_id, |room| {
                                                match &mut room.player_status {
                                                    PlayerStatus::Paused(time)
                                                    | PlayerStatus::Playing(time) => *time = *val,
                                                }
                                            })
                                            .await;
                                        app_state
                                            .rooms
                                            .broadcast_msg_excluding(
                                                room_id,
                                                original_message,
                                                &[user_id],
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("Received msg decode error {err:#?}")
                }
            }
        }
        axum::extract::ws::Message::Ping(_) => {
            //ignore
        }
        axum::extract::ws::Message::Pong(_) => {
            //ignore
        }
        axum::extract::ws::Message::Close(_) => {
            info!("Received Close from socket disconnecting {user_id}");
            return true;
        }
    }
    return false;
}

impl IntoResponse for RoomJoinError {
    fn into_response(self) -> Response {
        match self {
            RoomJoinError::RoomProviderError(err) => match err {
                RoomProviderError::KeyGenerationFailed => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#?}")).into_response()
                }
                RoomProviderError::RoomDoesntExist => {
                    (StatusCode::BAD_REQUEST, format!("{err:#?}")).into_response()
                }
            },
        }
    }
}
