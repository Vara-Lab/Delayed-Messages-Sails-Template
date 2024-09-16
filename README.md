# Mini tamagotchi

When compiling (inside the wasm folder or in the root path), two files will be created, "app.idl" which specifies the types, services, etc; and "app_client.rs (contains all the necessary code) which will be used to communicate with this contract (receiver client), both files will be inside the "wasm" directory.

To upload the contract, you have to go to [IDEA](https://idea.gear-tech.io/programs?node=wss%3A%2F%2Ftestnet.vara.network) and upload the .opt.wasm (in target/wasm32-unknown-unknown/release) and idl files that were generated

To be able to communicate with this contract, you need to copy the "app_client.rs" on your contract files and use it in your code.

## Instructions to handle delayed messages in your contracts.

> This contract already implements delayed messages, you can change the logic of it. But you need to know some steps to achieve this.

## Setting up the clients

In your "app" directory, you need to create a module called "clients" where you can store the client for this contract. The directory tree would look like this: 

<pre>
    app
    ├── Cargo.toml <- file where dependencies (crates) are specified
    └── src <- Here you will find the contract files and directories
        ├── clients <- Directory where all services are stored
        |   ├── mod.rs <- file that specifies the contract clients module
        |   └── app_client.rs <- Client code used to communicate with the contract
        ├── services <- Directory where all services are stored
        |   ├── mod.rs <- file that specifies the contract services module
        |   └── contract_service.rs <- Contraxt service example
        ├── states <- Directory where all contract states are stored
        └── lib.rs <- file where the contract "program" is created
</pre>

In the mod.rs that is in "clients" directory, you need to "import" your client (to import clients, enums, etc):

```rust
// This is an example, the client will be created in "wasm" directory
pub mod app_client; 
```

## First contract setup

Sending a delayed message to the contract itself is a little tricky action, because sails needs you to encode the payload in a specific way ([Sails Payload Encoding](https://github.com/gear-tech/sails/blob/master/README.md#payload-encoding)).

To achieve this, the following steps must be followed:

> **Note**: You need to use the "ContractGasReservationsService" service that is in this contract to handle gas reservations, and before sending a delayed message, you must reserve gas so that it can be sent.

1. You need to "describe" your methods that the contract wwill use, including methods to be used for delayed messages. For delayed messages, you can only describe the method along with the data to return (You can change this later, it is necessary so that you can implement your contract logic along with the delayed messages), example:

```rust
// import necesary crates
use sails_rs::{
    prelude::*,
    gstd::msg
};
use gstd::exec::program_id;

// Creating the service struct
#[derive(Default)]
pub struct ServiceExample;

// Service methods
#[service]
impl ServiceExample {
    pub fn new() -> Self {
        Self
    }

    // Describe the method that wil send the delayed message
    pub fn send_delayed_message(&mut self) -> ServiceEvents {
        ServiceEvents::MessageWasSent
    }

    // Method to be called with the delayed message
    pub fn method_to_call_with_delayed_message(&mut self) ->  ServiceEvents {
        if msg::source() != program_id() {
            return ServiceEvents::OnlyContractCanDoThisAction;
        }

        // code ...

        ServiceEvents::MethodWasCalled
    }
}

// Enum for service events
#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ServiceEvents {
    OnlyContractCanDoThisAction,
    MessageWasSent,
    MethodWasCalled,
    NoReservationIdsToUnreserve
}
```

2. Then, you need to add the service in your contract "program".

```rust
#![no_std]

use sails_rs::prelude::*;

pub mod services; // Module of all services
pub mod states;   // Module of all services state
pub mod clients;  // Module of all clients

// import the necesary services
use services::{
    service_example::ServiceExample,
    // Service to handle gas reservations
    gas_reservation_service::ContractGasReservationsService,
};

#[derive(Default)]
pub struct ProgramExample;

#[program]
impl ProgramExample {
    pub fn new(tamagotchi_name: Option<String>) -> Self {
        // Seed to initiate gas reservations service state
        ContractGasReservationsService::seed();

        Self
    }

    // Route for gas reservation service
    #[route("ContractGasReservation")]
    pub fn contract_gas_reservation_svc(&self) -> ContractGasReservationsService {
        ContractGasReservationsService::new()
    }

    // Route for the service example
    #[route("TestingService")]
    pub fn service_example_svc(&self) -> ServiceExample {
        ServiceExample::new()
    }
}
```

3. Then, in the file 'build.rs' that is in the "wasm" directory, you need to add the next lines (If you modify the name of the program in this contract, you have to change the name in this file too):

```rust
use app::ProgramExample;
use sails_idl_gen::program;
use sails_client_gen::ClientGenerator;
use std::{env, path::PathBuf};

fn main() {
    // Build contract to get .opt.wasm
    gear_wasm_builder::build();

    // Path where the file "Cargo.toml" is located (points to the root of the project)
    // 'CARGO_MANIFEST_DIR' specifies this directory in en::var
    let cargo_toml_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Path where the file "app.idl" will be created
    let idl_path = cargo_toml_path.clone().join("app.idl");

    // This generate the contract IDL
    program::generate_idl_to_file::<ProgramExample>(idl_path.clone())
        .unwrap();

    // Generator of the clients of the contract
    ClientGenerator::from_idl_path(&idl_path)
        .with_mocks("with_mocks")
        .generate_to(cargo_toml_path.join("app_client.rs"))
        .unwrap();
}
```

4. Now you need to compile your contract to get the client of this contract

```sh
cargo build --release
```

## Setting delayed messages for the contract itself

1. Once you have compiled your contract, you will find the file "app_client.rs" in the "wasm" directory, you have to move this file in the "clients" directory, that is in "app/src/" (you can change the name of the file).

2. Once you have move it in the directory, you need to put the next line in the "mod.rs" file:

```rust
pub mod app_client;
```

3. And now, in your service, you can use the client to get the functions that enconde the "call" (including arguments if any). You have to import the gas reservation service to get reservations ids, and the client to encode the payload to send to the contract (in this case, the contract itself).

```rust
//code ...

// We put the route of the service "TestingService", so the name of the 
// client to use will be "testing_service"
use crate::clients::app_client::testing_service;
// Service of gas reservation to get reservations id
use crate::services::gas_reservation_service::ContractGasReservationsService;

// code ...
impl ServiceExample {
    // code... 

    // Describe the method that wil send the delayed message
    pub fn send_delayed_message(&mut self) -> ServiceEvents {
        // Before calling this action, we need to reserve gas in the 
        // gas reservation service
        let reservation_id = ContractGasReservationsService::state_mut()
            .get_reservation_id();

        // Check if the method returns a reservation id
        let Some(reservation_id) = reservation_id else {
            return ServiceEvents::NoReservationIdsToUnreserve;
        };

        // Encoded payload to send
        let payload = testing_service::io::MethodToCallWithDelayedMessage::encode_call();

        // sending the delayed message 
        msg::send_bytes_delayed_from_reservation(
            reservation_id, 
            program_id(), 
            payload, 
            0, 
            60, // two minutes (60 blocks)
        ).expect("Error while sending delayed message");

        ServiceEvents::MessageWasSent
    }

}

// Enum for service events (added new variant)
#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ServiceEvents {
    OnlyContractCanDoThisAction,
    MessageWasSent,
    MethodWasCalled,
    NoReservationIdsToUnreserve
}
```

4. If you notice, we add a new variant to the event enum service, so, you need to compile again your program, delete the current client and paste the new client in the "clients" directory, and compile again to upload in the [Gear IDEA](https://idea.gear-tech.io/programs?node=wss%3A%2F%2Ftestnet.vara.network), in order to avoid bugs, this is the tricky part. 

## Recommendations

- Before each call to a method that performs a delayed message, you have to reserve gas.
- You need to analyze how much gas is consumed in the call in order to reserve what is necessary.
- If you need to send a delayed message to other contract, and you update some enums, etc., in that contract, yo need to replace the old client of that contract to the newer, to avoid bugs or errors (like the final step in [Setting delayed messages for the contract itself](#setting-delayed-messages-for-the-contract-itself))
- The gas that you will use to reserve is in vara notation:
    * One vara is equal to: 1000000000000.
    * To reserve 0.1 varas of gas, you have to enter: 100000000000













