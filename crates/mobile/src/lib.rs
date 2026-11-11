//! A ponte com o app Android: uma função JNI só. O app passa banco, porta e token por
//! variáveis de ambiente (`Os.setenv`) antes de chamar, e o servidor lê com `Config::env`,
//! igual ao serviço do PC. Sem a crate `jni`: os dois ponteiros que a JVM manda não são usados.

use std::{ffi::c_void, sync::OnceLock};

use server::{Config, spawn};

/// Sobe o servidor (uma vez por processo) e devolve a porta, ou -1 se não deu.
/// Chamar de novo, quando a activity é recriada, devolve a mesma porta.
#[unsafe(no_mangle)]
pub extern "system" fn Java_app_mindblow_android_Node_start(_env: *mut c_void, _this: *mut c_void) -> i32 {
    static PORT: OnceLock<i32> = OnceLock::new();
    *PORT.get_or_init(|| match spawn(Config::env()) {
        Ok(a) => i32::from(a.port()),
        Err(e) => {
            eprintln!("mindblow: {e}");
            -1
        }
    })
}
