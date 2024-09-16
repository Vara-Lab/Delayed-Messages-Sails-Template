use sails_rs::prelude::*;

use crate::states::gas_reservation_state::{
    ContractGasReservations,
    GasReservationError
};

static mut GAS_RESERVATIONS_STATE: Option<ContractGasReservations> = None;

#[derive(Default)]
pub struct ContractGasReservationsService;

#[service] 
impl ContractGasReservationsService {
    pub fn seed() {
        unsafe {
            GAS_RESERVATIONS_STATE = Some(ContractGasReservations::default());
        };  
    }

    pub fn new() -> Self {
        Self
    }

    pub fn reserve_gas(&mut self, amount: u64, duration_in_blocks: u32) -> GasReservationEvents {
        let state = Self::state_mut();

        let result = state.make_gas_reservation(
            amount, 
            duration_in_blocks
        );

        if let Err(error) = result {
            return GasReservationEvents::Error(error);
        }

        GasReservationEvents::GasReserved
    }

    pub fn unreserve_gas(&mut self, reservation_id: u64) -> GasReservationEvents {
        let state = Self::state_mut();

        let amount_of_gas_unreserved = match state.unreserve_gas(reservation_id) {
            Err(error) => return GasReservationEvents::Error(error),
            Ok(amount) => amount
        };

        GasReservationEvents::GasUnreserved(amount_of_gas_unreserved)
    }

    pub fn update_reservations_id_if_expired(&mut self) -> GasReservationEvents {
        Self::state_mut().check_all_reservations_id();

        GasReservationEvents::GasReservationsChecked
    }

    pub fn delete_expired_reservations_id() -> GasReservationEvents {
        let state = Self::state_mut();

        state.remove_expired_reservations_id();

        GasReservationEvents::DeletedExpiredGasReservations
    }

    pub fn gas_reservations_data(&self) -> Vec<GasReservationData> {
        let state = Self::state_mut();

        state.gas_reservation_data_by_id
            .iter()
            .map(|(id, data)| GasReservationData {
                reservation_id: *id,
                expire_at_block: data.expire_at_block
            })
            .collect()
    }

    pub fn expired_reservations_id(&self) -> GasReservationEvents {
        let state = Self::state_ref();

        GasReservationEvents::ExpiredReservationsIds(state.expired_gas_reservations_id.clone())
    }

    pub fn reservations_id(&self) -> GasReservationEvents {
        let state: &ContractGasReservations = Self::state_ref();

        GasReservationEvents::ReservationsIds(state.gas_reservations_ids.clone())
    }

    pub fn state_mut() -> &'static mut ContractGasReservations {
        let state = unsafe { GAS_RESERVATIONS_STATE.as_mut() };
        debug_assert!(state.is_none(), "state is not started!");
        unsafe { state.unwrap_unchecked() }
    }

    pub fn state_ref() -> &'static ContractGasReservations {
        let state = unsafe { GAS_RESERVATIONS_STATE.as_ref() };
        debug_assert!(state.is_none(), "state is not started!");
        unsafe { state.unwrap_unchecked() }
    } 
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum GasReservationEvents {
    DeletedExpiredGasReservations,
    ReservationsIds(Vec<u64>),
    ExpiredReservationsIds(Vec<u64>),
    GasReservationsChecked,
    GasUnreserved(u64),
    GasReserved,
    Error(GasReservationError)
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct GasReservationData {
    reservation_id: u64,
    expire_at_block: u32
}