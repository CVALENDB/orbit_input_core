//! Este módulo centraliza las teclas básicas disponibles en un teclado
//! y define el `KeyCode` usado por el runtime de input.
//!
//! Aún no soporta combinaciones (SHIFT + KEY, CTRL + KEY, ALTGR + KEY).
//! Su soporte es **básico**: únicamente teclas comunes, alfanuméricas,
//! de función y control general.
//!
//! # Convenciones de tipos genéricos
//!
//! - **`KeyExt<B, N>`**: Conversión de teclas
//!   - `B` = **B**ackend (tipo nativo del sistema o del backemd usado)
//!   - `N` = **N**ormalized (tipo normalizado del runtime)
//!
//! - **`KeyStateExt<I, O>`**: Conversión de estados
//!   - `I` = **I**nput (estado externo/nativo)
//!   - `O` = **O**utput (estado interno/normalizado)
//!
//! # Ejemplos
//!
//! ## Uso básico (features default/windows/linux)
//! ```rust,ignore
//! use orbit_input_core::keyboard::{KeyCode, KeyState};
//! 
//! let key = KeyCode::A;
//! let state = KeyState::Pressed;
//! ```
//!
//! ## Implementación custom (feature = "raw")
//! ```rust,ignore
//! use orbit_input_core::keyboard::{KeyExt, KeyStateExt};
//! 
//! // Implementa los traits para tu backend personalizado
//! impl KeyExt<MyBackendKey, KeyCode> for MyBackendKey { ... }
//! impl KeyStateExt<MyBackendState, KeyState> for MyBackendState { ... }
//! ```


/// El trait [`KeyExt`] define la interfaz para **convertir entre códigos de tecla nativos**
/// y una representación unificada, ya sea el [`KeyCode`] del crate o uno definido por el usuario.
///
/// # Propósito
/// Este trait abstrae el proceso de traducción entre las teclas específicas de un backend
/// (por ejemplo, `evdev::Key` en Linux o `u16` en Windows)
/// y el tipo de tecla que un *runtime* desea utilizar internamente.
///
/// Está diseñado no solo para los features oficiales (`linux`, `windows`),
/// sino también para permitir a desarrolladores externos definir:
///
/// - **Su propio backend** (por ejemplo, una librería personalizada de input).
/// - **Su propio tipo de tecla normalizado**, distinto de [`KeyCode`].
///
/// De esta forma, cualquier *runtime* dentro del ecosistema de Orbit Engine
/// puede integrar su propio sistema de input, siempre que implemente este trait.
///
/// # Parámetros genéricos
/// - `B`: Tipo de tecla **nativo del backend** (por ejemplo, `evdev::Key`, `u16`, etc.).
/// - `N`: Tipo de tecla **normalizado** que se usará en el runtime (por ejemplo, [`KeyCode`]
///   u otro definido por el usuario).
///
/// # Ejemplo: backend personalizado
/// ```rust,ignore
/// use orbit_input_core::keyboard::KeyExt;
///
/// /// Tipo de tecla personalizado.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum MyKeyCode {
///     A, B, C,
///     Unknown,
/// }
///
/// /// Tipo de backend hipotético.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum MyBackendKey {
///     BtnA, BtnB, BtnC,
///     None,
/// }
///
/// impl KeyExt<MyBackendKey, MyKeyCode> for MyBackendKey {
///     fn from_backend_key(key: MyBackendKey) -> MyKeyCode {
///         match key {
///             MyBackendKey::BtnA => MyKeyCode::A,
///             MyBackendKey::BtnB => MyKeyCode::B,
///             MyBackendKey::BtnC => MyKeyCode::C,
///             _ => MyKeyCode::Unknown,
///         }
///     }
///
///     fn to_backend_key(code: MyKeyCode) -> MyBackendKey {
///         match code {
///             MyKeyCode::A => MyBackendKey::BtnA,
///             MyKeyCode::B => MyBackendKey::BtnB,
///             MyKeyCode::C => MyBackendKey::BtnC,
///             _ => MyBackendKey::None,
///         }
///     }
/// }
/// ```
///
/// # Implementaciones esperadas
/// - **Windows:** Traducción entre `u16` o `winuser::VK_*` ↔ [`KeyCode`].
/// - **Linux:** Traducción entre `evdev::Key` ↔ [`KeyCode`].
/// - **Custom:** Cualquier par de tipos que sigan el mismo contrato.
///
/// # Reglas de implementación
/// - Las conversiones deben ser **deterministas** y **simétricas** (cuando sea posible).
/// - El trait debe poder implementarse en entornos sin `std` (idealmente `no_std`).
/// - No debe realizar asignaciones dinámicas o conversiones costosas.
///
/// # Beneficios
/// - Permite diseñar *runtimes* de input completamente personalizados.
/// - Fomenta la independencia entre el motor y las implementaciones específicas de SO.
/// - Facilita la integración con sistemas embebidos, dispositivos externos o simuladores.
///
/// # Ejemplo simplificado (Linux)
/// ```rust,ignore
/// use evdev::Key as EvdevKey;
/// use orbit_input_core::keyboard::{KeyCode, KeyExt};
///
/// impl KeyExt<EvdevKey, KeyCode> for EvdevKey {
///     fn from_backend_key(key: EvdevKey) -> KeyCode {
///         match key {
///             EvdevKey::KEY_A => KeyCode::A,
///             EvdevKey::KEY_B => KeyCode::B,
///             _ => KeyCode::Unknown,
///         }
///     }
///
///     fn to_backend_key(code: KeyCode) -> EvdevKey {
///         match code {
///             KeyCode::A => EvdevKey::KEY_A,
///             KeyCode::B => EvdevKey::KEY_B,
///             _ => EvdevKey::KEY_RESERVED,
///         }
///     }
/// }
/// ```
pub trait KeyExt<B, N>
where
    B: Copy + PartialEq,
    N: Copy + PartialEq,
{
    /// Convierte una tecla del backend (`B`) a su representación normalizada (`N`).
    fn from_backend_key(key: B) -> N;

    /// Convierte una tecla normalizada (`N`) a su equivalente nativo del backend (`B`).
    fn to_backend_key(code: N) -> B;
}

/// El trait [`KeyStateExt`] define la interfaz para **traducir entre los estados de tecla nativos**
/// de un backend y una representación unificada o personalizada dentro del motor.
///
/// # Propósito
/// Este trait permite que los distintos *runtimes* (por ejemplo, `orbit_input_linux`,
/// `orbit_input_windows`, o incluso librerías de terceros) puedan convertir sus códigos
/// o enumeraciones nativas de estado de tecla a un formato estandarizado — como [`KeyState`] —
/// sin depender directamente del sistema operativo o API específica.
///
/// Al igual que [`KeyExt`], este trait busca mantener el **desacoplamiento total**
/// entre los backends nativos y la lógica interna del runtime.
///
/// # Parámetros genéricos
/// - `I`: Tipo de estado **externo o nativo (Input)**, proveniente del backend (por ejemplo, `evdev::KeyState` o `u32`).
/// - `O`: Tipo de estado **interno o normalizado (Output)**, usado por el runtime (por ejemplo, [`KeyState`]
///   u otro tipo definido por el usuario).
///
/// # Ejemplo: implementación para Linux (`evdev`)
/// ```rust,ignore
/// use evdev::KeyState as EvState;
/// use orbit_input_core::keyboard::{KeyState, KeyStateExt};
///
/// impl KeyStateExt<EvState, KeyState> for EvState {
///     fn from_external_state(state: EvState) -> KeyState {
///         match state {
///             EvState::Pressed => KeyState::Pressed,
///             EvState::Released => KeyState::Release,
///             _ => KeyState::Active, // algunos backends envían repetición o autorepeat
///         }
///     }
///
///     fn to_external_state(state: KeyState) -> EvState {
///         match state {
///             KeyState::Pressed => EvState::Pressed,
///             KeyState::Release => EvState::Released,
///             KeyState::Active => EvState::Pressed, // opcionalmente mapeado a "hold"
///         }
///     }
/// }
/// ```
///
/// # Ejemplo: implementación hipotética en Windows
/// ```rust,ignore
/// use orbit_input_core::keyboard::{KeyState, KeyStateExt};
///
/// impl KeyStateExt<u32, KeyState> for u32 {
///     fn from_external_state(state: u32) -> KeyState {
///         match state {
///             0x0001 => KeyState::Pressed,
///             0x0002 => KeyState::Active,
///             0x0003 => KeyState::Release,
///             _ => KeyState::Release,
///         }
///     }
///
///     fn to_external_state(state: KeyState) -> u32 {
///         match state {
///             KeyState::Pressed => 0x0001,
///             KeyState::Active  => 0x0002,
///             KeyState::Release => 0x0003,
///         }
///     }
/// }
/// ```
///
/// # Reglas de implementación
/// - Las conversiones deben ser **deterministas** y **simétricas** siempre que sea posible.
/// - No deben involucrar asignaciones dinámicas o lógica costosa.
/// - El trait debe poder implementarse en entornos **sin `std`**.
/// - Puede utilizarse con cualquier tipo de representación, incluso numérica o binaria.
///
/// # Beneficios
/// - Permite que cada backend traduzca sus propios códigos de estado.
/// - Mantiene el motor desacoplado del sistema operativo.
/// - Facilita la interoperabilidad entre distintas APIs de entrada.
/// - Es totalmente extensible: los usuarios pueden definir sus propios tipos `I` y `O`.
pub trait KeyStateExt<I, O>
where 
    I: Copy + PartialEq,
    O: Copy + PartialEq,
{
    /// Convierte un estado nativo del backend (`I`) a su representación interna (`O`).
    fn from_external_state(state: I) -> O;

    /// Convierte un estado interno (`O`) a su equivalente nativo del backend (`I`).
    fn to_external_state(state: O) -> I;
}