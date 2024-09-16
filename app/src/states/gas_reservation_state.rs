use sails_rs::{
    prelude::*,
    collections::HashMap
};
use gstd::{
    exec::block_height, ReservationId, ReservationIdExt
};

// const RESERVATION_AMOUNT: u64 = 100_000_000_000; // 0.05 varas
// const RESERVATION_BLOCKS_DURATION: u32 = 1; // One block

#[derive(Default)]
pub struct ContractGasReservations {
    pub current_gas_reservation_id: u64,
    pub gas_reservations_ids: Vec<u64>,
    pub expired_gas_reservations_id: Vec<u64>,
    pub gas_reservation_data_by_id: HashMap<u64, GasReservationData>,
}


impl ContractGasReservations {
    /// ## Make a gas reservation
    pub fn make_gas_reservation(
        &mut self, 
        reservation_amount: u64,
        reservation_duration_in_blocks: u32
    ) -> Result<(), GasReservationError> {
        // Get current block height to know later if 
        // reserved id still works
        let current_block_height = block_height();

        // Get new reservation id
        let reserved_gas_id = ReservationId::reserve(
            reservation_amount, 
            reservation_duration_in_blocks
        )
        .map_err(|_| 
            GasReservationError::ErrorWhileDoingReservation
        )?;

        // Get current reservation id and checks if exists overflow
        let current_reservation_id = self.current_gas_reservation_id
            .checked_add(1)
            .ok_or(GasReservationError::GasReservationIdOverflow)?;
    
        // Create new reservation data to save in state
        let reservation_data = GasReservationData::new(
            reserved_gas_id, 
            current_block_height + reservation_duration_in_blocks
        );

        // update current reservation id
        let reservation_id_to_use = self.current_gas_reservation_id;

        // Save reservation data
        self.gas_reservation_data_by_id
            .insert(reservation_id_to_use, reservation_data);

        // Save reservation id in valid reservations id vector
        self.gas_reservations_ids
            .push(reservation_id_to_use);
        
        // Save current reservation id
        self.current_gas_reservation_id = current_reservation_id;

        Ok(())
    }

    /// ## Unreserve gas reservation
    /// If the gas reservation id is valid, the method will unreserve the gas
    pub fn unreserve_gas(&mut self, reservation_id: u64) -> Result<u64, GasReservationError> {
        // Check if the contract contains reservations id or if the reservation id exists
        if self.gas_reservations_ids.is_empty() || self.gas_reservation_data_by_id.contains_key(&reservation_id) {
            return Err(GasReservationError::NoReservationIdsToUnreserve);
        }

        // Remove the reservation id from stored gas reserved data
        let expired_reservation_data = self.gas_reservation_data_by_id
            .remove(&reservation_id)
            .unwrap();

        // remove the reservation id from the available reservations id
        let reservations_id_updated = self.gas_reservations_ids
            .iter()
            .filter_map(|id| (*id != reservation_id).then(|| *id))
            .collect();

        // update the available reservations id
        self.gas_reservations_ids = reservations_id_updated;

        // check the result of the unreserve the gas
        let result = ReservationId::unreserve(expired_reservation_data.reserved_gas_id)
            .map_err(|_| GasReservationError::UnableToUnreserveGas)?;

        Ok(result)
    }

    /// ## Get a reservarvation id to use reserved gas
    /// Returns an option, none if there is no valid reservations id,
    /// if it is expired, it will save the expired reservations id
    pub fn get_reservation_id(&mut self) -> Option<ReservationId> {
        let reservation_data = loop {
            let temp = self.gas_reservations_ids.pop();
            let Some(reservation_id) = temp else {
                break None;
            };

            // Get reservation data
            let reservation_data = self.gas_reservation_data_by_id
                .get(&reservation_id)
                .unwrap();

            // Check if the reserved gas is expired
            let reservation_is_expired = ContractGasReservations::gas_reservation_id_is_expired(reservation_data);
            
            if reservation_is_expired {
                // if is expired, save the reservation id in expired reservations vector
                self.expired_gas_reservations_id.push(reservation_id);
            } else {
                // The gas reserved is valid, remove from HashMap
                let temp = self.gas_reservation_data_by_id
                    .remove(&reservation_id)
                    .unwrap();
                
                // Returns the reserved gas id
                break Some(temp.reserved_gas_id);
            }
        };

        // Returns reserved gas id
        reservation_data
    }

    /// ## Update reservations id
    /// Check all vouchers id, if is expired, it will move expired voucher id
    /// to the expired vouchers id vector
    pub fn check_all_reservations_id(&mut self) {
        if self.gas_reservations_ids.is_empty() {
            return;
        }

        let mut expired_reservations_id = Vec::new();
        let mut alive_reservations_id = Vec::new();

        for reservation_id in self.gas_reservations_ids.iter() {
            let reservation_data = self.gas_reservation_data_by_id
                .get(reservation_id)
                .unwrap();

            if Self::gas_reservation_id_is_expired(reservation_data) {
                expired_reservations_id.push(*reservation_id);
            } else {
                alive_reservations_id.push(*reservation_id);
            }
        }

        self.gas_reservations_ids = alive_reservations_id;
        self.expired_gas_reservations_id.append(&mut expired_reservations_id);
    }

    /// ## Delete expired reservations
    /// Deled all reservations id from the contract
    pub fn remove_expired_reservations_id(&mut self) {
        if self.expired_gas_reservations_id.len() < 1 {
            return;
        }

        self.expired_gas_reservations_id.iter().for_each(|id| {
            self.gas_reservation_data_by_id.remove(id);            
        }); 

        self.expired_gas_reservations_id.clear();
    }
    
    pub fn gas_reservation_id_is_expired(reservation_data: &GasReservationData) -> bool {
        let actual_block_height = block_height();
        
        actual_block_height > reservation_data.expire_at_block
    }
}

#[derive(Clone)]
pub struct GasReservationData {
    pub reserved_gas_id: ReservationId,
    pub expire_at_block: u32
}

impl GasReservationData {
    pub fn new(
        reserved_gas_id: ReservationId,
        expire_at_block: u32,
    ) -> Self {
        Self {
            reserved_gas_id,
            expire_at_block
        }
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum GasReservationError {
    NoReservationIdsToUnreserve,
    UnableToUnreserveGas,
    ErrorWhileDoingReservation,
    GasReservationIdOverflow,
    GasReservationIsExpired(ReservationId),
    NoGasReservationsInContract
}
