use crate::{
    action::Action,
    common::Duration,
    state::State,
    task::{NextRun, Task},
};

pub enum ChargerAction {
    Hold,
    Charge,
    // Turns out NiMH doesn't suffer from memory effect.
    // Discharge,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum BatteryState {
    /// Battery voltage is too low. Stop discharge.
    Critical,
    BelowNominal,
    AboveNominal,
    // Charging is in progress.
    Charging,
    // Charging has been completed.
    Charged,
}

#[derive(Default)]
pub struct Charger {
    bat_voltage: (f32, f32),
    state: ChargerState,
}

enum ChargerState {
    Hold,
    Charge,
    Charged,
}

impl Default for ChargerState {
    fn default() -> Self {
        Self::Hold
    }
}

impl Task<State, Action> for Charger {
    fn run(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        match self.state {
            ChargerState::Hold => self.run_hold(state),
            ChargerState::Charge => self.run_charge(state),
            ChargerState::Charged => self.run_charged(state),
        }
    }
}

impl Charger {
    fn enter(&mut self, state: ChargerState) -> Option<ChargerAction> {
        match state {
            ChargerState::Hold => {
                self.state = ChargerState::Hold;
                Some(ChargerAction::Hold)
            }
            ChargerState::Charge => {
                self.state = ChargerState::Charge;
                Some(ChargerAction::Charge)
            }
            ChargerState::Charged => {
                self.state = ChargerState::Charged;
                Some(ChargerAction::Hold)
            }
        }
    }

    fn run_hold(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        self.bat_voltage = state.bat_voltage;

        state.bat_level = Self::bat_level(state.bat_voltage);

        let (v1, v2) = state.bat_voltage;
        let action = if state.ext_power && v1 < NIMH_HIGH && v2 < NIMH_HIGH {
            self.enter(ChargerState::Charge)
        } else {
            None
        };

        (
            action.map(Action::Battery),
            NextRun::After(Duration::from_ticks(5_000_000)),
        )
    }

    fn run_charge(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        let (d1, d2) = diff2f(state.bat_voltage, self.bat_voltage);
        self.bat_voltage = max2f(state.bat_voltage, self.bat_voltage);

        state.bat_level = BatteryState::Charging;

        let action = if !state.ext_power {
            self.enter(ChargerState::Hold)
        } else {
            let (v1, v2) = state.bat_voltage;
            let is_adc_saturated = v1 + v2 >= 2.99;
            if d1 <= NIMH_NDV || d2 <= NIMH_NDV || is_adc_saturated {
                self.enter(ChargerState::Charged)
            } else {
                None
            }
        };

        (
            action.map(Action::Battery),
            NextRun::After(Duration::from_ticks(5_000_000)),
        )
    }

    fn run_charged(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        self.bat_voltage = state.bat_voltage;

        state.bat_level = BatteryState::Charged;

        let action = if !state.ext_power {
            self.enter(ChargerState::Hold)
        } else {
            let (v1, v2) = state.bat_voltage;
            if v1 < NIMH_HIGH || v2 < NIMH_HIGH {
                self.enter(ChargerState::Charge)
            } else {
                None
            }
        };

        (
            action.map(Action::Battery),
            NextRun::After(Duration::from_ticks(5_000_000)),
        )
    }

    fn bat_level(bat_voltage: (f32, f32)) -> BatteryState {
        match bat_voltage {
            (v1, v2) if v1 >= NIMH_MPV && v2 >= NIMH_MPV => BatteryState::AboveNominal,
            (v1, v2) if v1 >= NIMH_EODV && v2 >= NIMH_EODV => BatteryState::BelowNominal,
            _ => BatteryState::Critical,
        }
    }
}

fn max2f(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (
        if a.0 >= b.0 { a.0 } else { b.0 },
        if a.1 >= b.1 { a.1 } else { b.1 },
    )
}

fn diff2f(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 - b.0, a.1 - b.1)
}

// Do not initiate charging, if the battery voltage is above this level.
pub const NIMH_HIGH: f32 = 1.35;
// Stop charging, if the battery voltage drops more than the negative dV threshold under charging.
pub const NIMH_NDV: f32 = -0.002;
// NiMH battery mid-point voltage.
pub const NIMH_MPV: f32 = 1.25;
// NiMH battery end of discharge voltage.
const NIMH_EODV: f32 = 0.9;
