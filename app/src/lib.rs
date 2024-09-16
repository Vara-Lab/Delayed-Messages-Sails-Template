#![no_std]

use sails_rs::prelude::*;

pub mod services;
pub mod states;
pub mod clients;

use services::{
    mini_tamagotchi_service::MiniTamagotchiService,
    gas_reservation_service::ContractGasReservationsService,
};

#[derive(Default)]
pub struct MiniTamagotchiProgram;

#[program]
impl MiniTamagotchiProgram {
    pub fn new(tamagotchi_name: Option<String>) -> Self {
        match tamagotchi_name {
            Some(name) => {
                MiniTamagotchiService::seed(name);
            },
            _ => {
                MiniTamagotchiService::seed("no name".to_string());
            }
        }
        ContractGasReservationsService::seed();

        Self
    }

    #[route("MiniTamagotchi")]
    pub fn mini_tamagotchi_svc(&self) -> MiniTamagotchiService {
        MiniTamagotchiService::new()
    }

    #[route("ContractGasReservation")]
    pub fn contract_gas_reservation_svc(&self) -> ContractGasReservationsService {
        ContractGasReservationsService::new()
    }
}