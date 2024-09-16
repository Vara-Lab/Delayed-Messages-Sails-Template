use sails_rs::{
    prelude::*,
    gstd::msg
};
use gstd::exec::program_id;

use crate::clients::self_app_client::mini_tamagotchi;
use crate::states::mini_tamagotchi_state::MiniTamagotchi;
use crate::services::gas_reservation_service::ContractGasReservationsService;

static mut MINI_TAMAGOTCHI_STATE: Option<MiniTamagotchi> = None;

#[derive(Default)]
pub struct MiniTamagotchiService;

#[service]
impl MiniTamagotchiService {

    pub fn seed(name: String) {
        unsafe {
            MINI_TAMAGOTCHI_STATE = Some(MiniTamagotchi::new(name));
        };
    }

    pub fn new() -> Self {
        Self
    }

    pub fn change_name(&mut self, name: String) -> MiniTamagotchiEvents {
        Self::state_mut().name = name;
        MiniTamagotchiEvents::TamagotchiNameChanged
    }

    pub fn stop_playing(&mut self) -> MiniTamagotchiEvents {
        let caller = msg::source();

        if caller != program_id() {
            return MiniTamagotchiEvents::OnlyContractCanSendThisMessage;
        }

        Self::state_mut().is_playing = false;

        MiniTamagotchiEvents::TamagotchiStopPlaying
    }

    pub fn send_tamagotchi_to_play(&mut self) -> MiniTamagotchiEvents {
        let state = Self::state_mut();

        if state.is_playing {
            return MiniTamagotchiEvents::TamagotchiIsPlaying;
        }

        let reservation_id = ContractGasReservationsService::state_mut()
            .get_reservation_id();

        let Some(reservation_id) = reservation_id else {
            return MiniTamagotchiEvents::NoReservationIdsToUnreserve;
        };

        let payload = mini_tamagotchi::io::StopPlaying::encode_call();

        msg::send_bytes_delayed_from_reservation(
            reservation_id, 
            program_id(), 
            payload, 
            0, 
            60, // two minutes (60 blocks)
        ).expect("Error while sending delayed message");

        state.is_playing = true;

        // Logic for delayed messages

        MiniTamagotchiEvents::TamagotchiIsPlaying
    }

    pub fn set_tamagotchi_is_hungry(&mut self) -> MiniTamagotchiEvents {
        let caller = msg::source();

        if caller != program_id() {
            return MiniTamagotchiEvents::OnlyContractCanSendThisMessage;
        }

        Self::state_mut().is_hungry = true;

        MiniTamagotchiEvents::TamagotchiIsHungry
    }

    pub fn feed_tamagotchi(&mut self) -> MiniTamagotchiEvents {
        let state = Self::state_mut();

        if !state.is_hungry {
            return MiniTamagotchiEvents::TamagotchiIsNotHungry;
        }

        let reservation_id = ContractGasReservationsService::state_mut()
            .get_reservation_id();

        let Some(reservation_id) = reservation_id else {
            return MiniTamagotchiEvents::NoReservationIdsToUnreserve;
        };

        let payload = mini_tamagotchi::io::SetTamagotchiIsHungry::encode_call();

        msg::send_bytes_delayed_from_reservation(
            reservation_id, 
            program_id(), 
            payload, 
            0, 
            60, // two minutes (60 blocks)
        ).expect("Error while sending delayed message");
    
        state.is_hungry = false;

        MiniTamagotchiEvents::TamagotchiAteSomething
    }

    pub fn tamagotchi_name(&self) -> MiniTamagotchiEvents {
        let name = &Self::state_ref().name;

        MiniTamagotchiEvents::TamagotchiName(name.clone())
    }

    pub fn tamagotchi_is_playing(&self) -> MiniTamagotchiEvents {
        let state = Self::state_ref();

        if state.is_playing {
            MiniTamagotchiEvents::TamagotchiIsPlaying
        } else {
            MiniTamagotchiEvents::TamagotchiIsNotPlaying
        }
    }

    pub fn tamagotchi_is_hungry(&self) -> MiniTamagotchiEvents {
        let state = Self::state_ref();

        if state.is_hungry {
            MiniTamagotchiEvents::TamagotchiIsHungry
        } else {
            MiniTamagotchiEvents::TamagotchiIsNotHungry
        }
    }

    fn state_mut() -> &'static mut MiniTamagotchi {
        let state: Option<&mut MiniTamagotchi> = unsafe { MINI_TAMAGOTCHI_STATE.as_mut() };
        debug_assert!(state.is_none(), "state is not started!");
        unsafe { state.unwrap_unchecked() }
    }

    fn state_ref() -> &'static MiniTamagotchi {
        let state = unsafe { MINI_TAMAGOTCHI_STATE.as_ref() };
        debug_assert!(state.is_none(), "state is not started!");
        unsafe { state.unwrap_unchecked() }
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MiniTamagotchiEvents {
    MessageDelayedSend,
    NoReservationIdsToUnreserve,
    OnlyContractCanSendThisMessage,
    TamagotchiName(String),
    TamagotchiNameChanged,
    TamagotchiStopPlaying,
    TamagotchiAteSomething,
    TamagotchiIsPlaying,
    TamagotchiIsNotPlaying,
    TamagotchiIsHungry,
    TamagotchiIsNotHungry
}