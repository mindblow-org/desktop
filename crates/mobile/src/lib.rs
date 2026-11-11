//! A ponte com o app Android: uma função JNI só. O app passa banco, porta e token por
//! variáveis de ambiente (`Os.setenv`) antes de chamar, e o servidor lê com `Config::env`,
//! igual ao serviço do PC. Sem a crate `jni`: os dois ponteiros que a JVM manda não são usados.

use std::{ffi::c_void, sync::Mutex};

use server::{Config, spawn};

/// Sobe o servidor e devolve a porta, ou -1 se não deu (porta ocupada, banco que não abre).
/// Depois que sobe, chamar de novo devolve a mesma porta; depois de uma falha, tenta outra vez.
#[unsafe(no_mangle)]
pub extern "system" fn Java_app_mindblow_android_Node_start(_env: *mut c_void, _this: *mut c_void) -> i32 {
    static PORT: Mutex<Option<u16>> = Mutex::new(None);
    let mut p = PORT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(port) = *p {
        return i32::from(port);
    }
    match spawn(Config::env()) {
        Ok(a) => {
            *p = Some(a.port());
            i32::from(a.port())
        }
        Err(e) => {
            eprintln!("mindblow: {e}");
            -1
        }
    }
}
