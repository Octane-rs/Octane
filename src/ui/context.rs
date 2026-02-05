use eframe::egui_wgpu::RenderState;

use crate::core::model::Model;
use crate::core::msg::Msg;
use crate::shell::app::{AppData, MsgSender};

pub struct ViewContext<'a> {
    pub model: &'a Model,
    pub data: &'a mut AppData,

    tx: &'a MsgSender,
}

#[derive(Clone)]
pub struct OwnedViewContext {
    pub state: RenderState,

    tx: MsgSender,
}

impl<'a> ViewContext<'a> {
    pub const fn new(model: &'a Model, data: &'a mut AppData, tx: &'a MsgSender) -> Self {
        Self { model, data, tx }
    }

    pub fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
    }
}

impl OwnedViewContext {
    pub const fn new(state: RenderState, tx: MsgSender) -> Self {
        Self { state, tx }
    }

    pub fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
    }
}
