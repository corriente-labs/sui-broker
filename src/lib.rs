#![deny(warnings)]

mod macros;

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    str::FromStr as _,
};

use sui_sdk::{
    crypto::{AccountKeystore, FileBasedKeystore},
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    SuiClient,
};

const SUI_RPC_HOST: &str = "http://127.0.0.1:5001";

/// # Safety
/// Pointers must be valid, and point to a null-terminated
/// string. What happens otherwise is UB.
///
/// # Panics
/// Panics if caller has provided a pointer that points to a invalid C
/// string with a NUL terminator of size equal or more than `isize::MAX`,
/// or points to a non-UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn evm(
    evm_id: *const c_char,
    evm_state_id: *const c_char,
    gas_id: *const c_char,
    signer: *const c_char,
) -> *mut c_char {
    let evm_id = CStr::from_ptr(evm_id).to_str().unwrap();
    let evm_id = try_from_str!(ObjectID, evm_id);

    let evm_state_id = CStr::from_ptr(evm_state_id).to_str().unwrap();
    let evm_state_id = try_from_str!(ObjectID, evm_state_id);

    let gas_id = CStr::from_ptr(gas_id).to_str().unwrap();
    let gas_id = try_from_str!(ObjectID, gas_id);

    let signer = CStr::from_ptr(signer).to_str().unwrap();
    let signer = try_from_str!(SuiAddress, signer);

    let keystore_path = if let Some(v) = dirs::home_dir() {
        v.join(".sui").join("sui_config").join("sui.keystore")
    } else {
        let str = "Failed to obtain home directory path";
        return CString::new(str).unwrap().into_raw();
    };
    let file_based_keystore = match FileBasedKeystore::load_or_create(&keystore_path) {
        Ok(file_based_keystore) => file_based_keystore,
        Err(err) => {
            let str = format!("Failed FileBasedKeystore: `{:?}`, {}", keystore_path, err);
            return CString::new(str).unwrap().into_raw();
        }
    };

    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            let str = format!("Failed tokio runtime: {}", err);
            return CString::new(str).unwrap().into_raw();
        }
    };

    let mut result: *mut c_char = std::ptr::null_mut();

    rt.block_on(async {
        let sui = match SuiClient::new_rpc_client(SUI_RPC_HOST, None).await {
            Ok(sui) => sui,
            Err(err) => {
                let str = format!("Failed new_rpc_client: `{}`, {}", SUI_RPC_HOST, err);
                result = CString::new(str).unwrap().into_raw();
                return;
            }
        };

        let arg0 = SuiJsonValue::from_str(&evm_state_id.to_string()).unwrap();
        let arg1 = SuiJsonValue::from_str("hello").unwrap();
        let call_call = match sui
            .transaction_builder()
            .move_call(
                signer,
                evm_id,
                "vm",
                "call",
                vec![],           // type args
                vec![arg0, arg1], // call args
                Some(gas_id),
                1000,
            )
            .await
        {
            Ok(call_call) => call_call,
            Err(err) => {
                let str = format!("Failed move_call: {}", err);
                result = CString::new(str).unwrap().into_raw();
                return;
            }
        };

        let signature = match file_based_keystore.sign(&signer, &call_call.to_bytes()) {
            Ok(signature) => signature,
            Err(err) => {
                let str = format!("Failed sign: {}", err);
                result = CString::new(str).unwrap().into_raw();
                return;
            }
        };

        let response = match sui
            .quorum_driver()
            .execute_transaction(Transaction::new(call_call, signature))
            .await
        {
            Ok(response) => response,
            Err(err) => {
                let str = format!("Failed execute_transaction: {}", err);
                result = CString::new(str).unwrap().into_raw();
                return;
            }
        };
        dbg!(response);
    });

    result
}
