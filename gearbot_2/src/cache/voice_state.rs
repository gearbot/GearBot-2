use twilight_model::id::ChannelId;
use twilight_model::voice::VoiceState as TwilightVoiceState;

pub struct VoiceState {
    pub connected_to: ChannelId,
    pub muted: bool,
    pub server_muted: bool,
    pub deafened: bool,
    pub server_deafened: bool,
    pub video: bool,
    pub streaming: bool,
}

impl VoiceState {
    pub fn from_state(state: TwilightVoiceState) -> Option<Self> {
        if state.channel_id.is_some() {
            Some(
                VoiceState {
                    connected_to: state.channel_id.unwrap(),
                    muted: state.self_mute,
                    server_muted: state.mute,
                    deafened: state.self_deaf,
                    server_deafened: state.deaf,
                    video: state.self_video,
                    streaming: state.self_stream,
                }
            )
        } else {
            None
        }
    }
}