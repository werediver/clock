use crate::features::charger::ChargerAction;

pub enum Action {
    Display(seg_disp::disp::Action),
    Battery(ChargerAction),
}
