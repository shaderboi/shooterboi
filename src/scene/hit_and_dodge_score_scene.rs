use chrono::NaiveDateTime;
use conrod_core::widget::envelope_editor::EnvelopePoint;
use conrod_core::widget::{Canvas, Text};
use conrod_core::{Labelable, Positionable, Sizeable, Widget};
use winit::event_loop::ControlFlow;

use crate::audio::AudioContext;
use crate::gui::ConrodHandle;
use crate::input_manager::InputManager;
use crate::renderer::Renderer;

use crate::database::Database;
use crate::scene::{
    MaybeMessage, Scene, SceneOp, BUTTON_HEIGHT, BUTTON_WIDTH, GAP_BETWEEN_ITEM, MARGIN,
};
use crate::window::Window;
use conrod_core::widget_ids;
use gluesql::data::Value;
use winit::event::VirtualKeyCode;

pub struct HitAndDodgeGameScoreDisplay {
    pub accuracy: f32,
    pub hit: u16,
    pub miss: u16,
    pub score: i32,
    pub avg_hit_time: f32,
    pub created_at: NaiveDateTime,
}

impl HitAndDodgeGameScoreDisplay {
    pub fn new() -> Self {
        Self {
            accuracy: 0.0,
            hit: 0,
            miss: 0,
            score: 0,
            avg_hit_time: 0.0,
            created_at: NaiveDateTime::from_timestamp(0, 0),
        }
    }

    pub fn read_message(&mut self, message: &MaybeMessage) {
        if let Some(msg) = message {
            self.hit = match *msg.get("hit").unwrap() {
                Value::I64(x) => x,
                _ => unreachable!(),
            } as u16;
            self.miss = match *msg.get("miss").unwrap() {
                Value::I64(x) => x,
                _ => unreachable!(),
            } as u16;
            self.score = match *msg.get("score").unwrap() {
                Value::I64(x) => x,
                _ => unreachable!(),
            } as i32;
            self.accuracy = match *msg.get("accuracy").unwrap() {
                Value::F64(x) => x,
                _ => unreachable!(),
            } as f32;
            self.created_at = match *msg.get("created_at").unwrap() {
                Value::Timestamp(x) => x,
                _ => unreachable!(),
            };
            self.avg_hit_time = match *msg.get("avg_hit_time").unwrap() {
                Value::F64(x) => x,
                _ => unreachable!(),
            } as f32;
            self.accuracy = self.hit as f32 / (self.hit + self.miss).max(1) as f32 * 100.0;
        }
    }
}

widget_ids! {
    pub struct HitAndDodgeScoreSceneIds {
        // The main canvas
        canvas,
        title_label,
        see_score_history_button,
        next_button,

        accuracy_canvas,
        accuracy_label,
        accuracy_value_label,

        hit_canvas,
        hit_label,
        hit_value_label,

        miss_canvas,
        miss_label,
        miss_value_label,

        score_canvas,
        score_label,
        score_value_label,

        avg_hit_time_canvas,
        avg_hit_time_label,
        avg_hit_time_value_label,
    }
}

pub struct HitAndDodgeScoreScene {
    ids: HitAndDodgeScoreSceneIds,
    score: HitAndDodgeGameScoreDisplay,
}

impl HitAndDodgeScoreScene {
    pub fn new(conrod_handle: &mut ConrodHandle) -> Self {
        let ids = HitAndDodgeScoreSceneIds::new(conrod_handle.get_ui_mut().widget_id_generator());
        Self {
            ids,
            score: HitAndDodgeGameScoreDisplay::new(),
        }
    }
}

impl Scene for HitAndDodgeScoreScene {
    fn init(
        &mut self,
        _message: MaybeMessage,
        _window: &mut Window,
        renderer: &mut Renderer,
        _conrod_handle: &mut ConrodHandle,
        _audio_context: &mut AudioContext,
        _database: &mut Database,
    ) {
        renderer.is_render_gui = true;
        renderer.is_render_game = false;

        // self.score.read_message(&message);
    }

    fn update(
        &mut self,
        _window: &mut Window,
        _renderer: &mut Renderer,
        input_manager: &InputManager,
        _delta_time: f32,
        conrod_handle: &mut ConrodHandle,
        _audio_context: &mut AudioContext,
        _control_flow: &mut ControlFlow,
        _database: &mut Database,
    ) -> SceneOp {
        let mut scene_op = SceneOp::None;

        let next_button;
        // let score_history_button;
        {
            let ropa_font_id = *conrod_handle.get_font_id_map().get("ropa").unwrap();
            let mut ui_cell = conrod_handle.get_ui_mut().set_widgets();

            Canvas::new().set(self.ids.canvas, &mut ui_cell);

            Text::new("Training Report")
                .font_id(ropa_font_id)
                .align_middle_x()
                .mid_top_with_margin_on(self.ids.canvas, MARGIN)
                .set(self.ids.title_label, &mut ui_cell);

            const ITEM_WIDTH: f64 = 400.0;

            Canvas::new()
                .down_from(self.ids.title_label, 50.0)
                .align_middle_x()
                .w(ITEM_WIDTH)
                .set(self.ids.accuracy_canvas, &mut ui_cell);

            Canvas::new()
                .down_from(self.ids.accuracy_canvas, GAP_BETWEEN_ITEM)
                .align_middle_x()
                .w(ITEM_WIDTH)
                .set(self.ids.hit_canvas, &mut ui_cell);

            Canvas::new()
                .down_from(self.ids.hit_canvas, GAP_BETWEEN_ITEM)
                .align_middle_x()
                .w(ITEM_WIDTH)
                .set(self.ids.score_canvas, &mut ui_cell);

            Canvas::new()
                .down_from(self.ids.score_canvas, GAP_BETWEEN_ITEM)
                .align_middle_x()
                .w(ITEM_WIDTH)
                .set(self.ids.avg_hit_time_canvas, &mut ui_cell);

            Canvas::new()
                .down_from(self.ids.avg_hit_time_canvas, GAP_BETWEEN_ITEM)
                .align_middle_x()
                .w(ITEM_WIDTH)
                .set(self.ids.miss_canvas, &mut ui_cell);

            Text::new("Accuracy")
                .font_id(ropa_font_id)
                .mid_left_of(self.ids.accuracy_canvas)
                .left_justify()
                .set(self.ids.accuracy_label, &mut ui_cell);

            // Text::new(&format!("{}", self.score.accuracy))
            //     .font_id(ropa_font_id)
            //     .mid_right_of(self.ids.accuracy_canvas)
            //     .left_justify()
            //     .set(self.ids.accuracy_value_label, &mut ui_cell);

            Text::new("Hit")
                .font_id(ropa_font_id)
                .mid_left_of(self.ids.hit_canvas)
                .left_justify()
                .set(self.ids.hit_label, &mut ui_cell);

            Text::new(&format!("{}", self.score.hit))
                .font_id(ropa_font_id)
                .mid_right_of(self.ids.hit_canvas)
                .left_justify()
                .set(self.ids.hit_value_label, &mut ui_cell);

            Text::new("Score")
                .font_id(ropa_font_id)
                .mid_left_of(self.ids.score_canvas)
                .left_justify()
                .set(self.ids.score_label, &mut ui_cell);

            Text::new(&format!("{}", self.score.score))
                .font_id(ropa_font_id)
                .mid_right_of(self.ids.score_canvas)
                .left_justify()
                .set(self.ids.score_value_label, &mut ui_cell);

            Text::new("Avg hit time")
                .font_id(ropa_font_id)
                .mid_left_of(self.ids.avg_hit_time_canvas)
                .left_justify()
                .set(self.ids.avg_hit_time_label, &mut ui_cell);

            // Text::new(&format!("{}", self.score.avg_hit_time))
            //     .font_id(ropa_font_id)
            //     .mid_right_of(self.ids.avg_hit_time_canvas)
            //     .left_justify()
            //     .set(self.ids.avg_hit_time_value_label, &mut ui_cell);

            Text::new("Miss")
                .font_id(ropa_font_id)
                .mid_left_of(self.ids.miss_canvas)
                .left_justify()
                .set(self.ids.miss_label, &mut ui_cell);

            Text::new(&format!("{}", self.score.miss))
                .font_id(ropa_font_id)
                .mid_right_of(self.ids.miss_canvas)
                .left_justify()
                .set(self.ids.miss_value_label, &mut ui_cell);

            next_button = conrod_core::widget::Button::new()
                .label("Next")
                .label_font_id(ropa_font_id)
                .bottom_right_with_margin_on(self.ids.canvas, MARGIN)
                .wh(conrod_core::Dimensions::new(BUTTON_WIDTH, BUTTON_HEIGHT))
                .set(self.ids.next_button, &mut ui_cell);

            // score_history_button = conrod_core::widget::Button::new()
            //     .label("See Score History")
            //     .label_font_id(ropa_font_id)
            //     .up_from(self.ids.next_button, 20.0)
            //     .wh(conrod_core::Dimensions::new(BUTTON_WIDTH, BUTTON_HEIGHT))
            //     .set(self.ids.see_score_history_button, &mut ui_cell);
        }

        if input_manager.is_keyboard_press(&VirtualKeyCode::Escape)
            || input_manager.is_keyboard_press(&VirtualKeyCode::Return)
            || input_manager.is_keyboard_press(&VirtualKeyCode::Space)
            || next_button.was_clicked()
        {
            scene_op = SceneOp::Pop(2, None);
        }

        scene_op
    }

    fn deinit(
        &mut self,
        _window: &mut Window,
        _renderer: &mut Renderer,
        _conrod_handle: &mut ConrodHandle,
        _audio_context: &mut AudioContext,
        _database: &mut Database,
    ) {
    }
}
