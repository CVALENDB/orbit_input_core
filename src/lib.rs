//! # Crate `orbit_input_core`
//!
//! **Protocolo de traits** para construir sistemas de input compatibles con **Orbit Engine**.
//!
//! Este crate define únicamente las **interfaces (traits)** necesarias para implementar
//! un runtime de entrada compatible con el ecosistema de Orbit Engine.  
//! **No contiene implementaciones concretas** — solo el contrato que deben cumplir.
//!
//! ---
//!
//! ## ¿Para quién es este crate?
//!
//! ### 👥 **Para usuarios finales de Orbit Engine**
//!
//! Si solo quieres **usar** el sistema de input en tu juego, **NO necesitas este crate**.  
//! Usa directamente la implementación oficial:
//!
//! ```toml
//! [dependencies]
//! orbit_input = "0.1"
//! ```
//!
//! ### 🔧 **Para implementadores de backends personalizados**
//!
//! Si quieres **crear tu propio runtime de input** compatible con Orbit Engine
//! (por ejemplo, para un dispositivo custom, sistema embebido, o un backend experimental),
//! **este es el crate que necesitas**:
//!
//! ```toml
//! [dependencies]
//! orbit_input_core = "0.1"
//! ```
//!
//! ---
//!
//! ## Filosofía de diseño
//!
//! `orbit_input_core` sigue el principio de **inversión de dependencias**:
//!
//! - **Orbit Engine depende de `orbit_input_core`** (los traits), no de implementaciones concretas
//! - **Cualquier runtime que implemente estos traits** es compatible con Orbit Engine
//! - Los usuarios pueden elegir entre la implementación oficial (`orbit_input`) o crear la suya
//!
//! Esto permite:
//! - ✅ Máxima flexibilidad para casos de uso especializados
//! - ✅ Testing más sencillo (mocks que implementan los traits)
//! - ✅ Soporte para dispositivos no estándar
//! - ✅ Backends experimentales sin modificar el motor
//!
//! ---
//!
//! ## Contenido del crate
//!
//! Este crate **solo define traits**, sin tipos concretos:
//!
//! ### Traits de conversión
//! - [`KeyExt<B, N>`]: Convierte entre teclas del backend nativo y teclas normalizadas
//! - [`KeyStateExt<I, O>`]: Convierte entre estados del backend y estados normalizados
//!
//! ### Traits de gestión de estado
//! - [`InputStateExt<K, S>`]: Interfaz para consultar el estado actual del input (frame actual)
//! - [`WithHistoryExt<K, S, T>`]: Extiende `InputStateExt` con sistema de historial temporal
//! - [`InputEvent`]: Representa un evento individual en el historial
//!
//! ---
//!
//! ## Ejemplo: Implementación básica
//!
//! ```rust,ignore
//! use orbit_input_core::{InputStateExt, KeyExt};
//! use std::collections::HashMap;
//!
//! // 1. Define tus propios tipos
//! #[derive(Copy, Clone, PartialEq, Eq, Hash)]
//! pub enum MyKeyCode { Jump, Crouch, Attack, Unknown }
//!
//! #[derive(Copy, Clone, PartialEq)]
//! pub enum MyKeyState { Down, Up }
//!
//! // 2. Implementa la conversión desde tu backend nativo
//! impl KeyExt<u32, MyKeyCode> for u32 {
//!     fn from_backend_key(native: u32) -> MyKeyCode {
//!         match native {
//!             0x20 => MyKeyCode::Jump,
//!             0x43 => MyKeyCode::Crouch,
//!             0x41 => MyKeyCode::Attack,
//!             _ => MyKeyCode::Unknown,
//!         }
//!     }
//!     
//!     fn to_backend_key(code: MyKeyCode) -> u32 {
//!         match code {
//!             MyKeyCode::Jump => 0x20,
//!             MyKeyCode::Crouch => 0x43,
//!             MyKeyCode::Attack => 0x41,
//!             MyKeyCode::Unknown => 0x00,
//!         }
//!     }
//! }
//!
//! // 3. Crea tu estructura de estado
//! pub struct MyInputSystem {
//!     keys: HashMap<MyKeyCode, MyKeyState>,
//! }
//!
//! // 4. Implementa el trait principal
//! impl InputStateExt<MyKeyCode, MyKeyState> for MyInputSystem {
//!     fn set_key(&mut self, key: MyKeyCode, state: MyKeyState) {
//!         self.keys.insert(key, state);
//!     }
//!     
//!     fn is_pressed(&self, key: MyKeyCode) -> bool {
//!         self.keys.get(&key) == Some(&MyKeyState::Down)
//!     }
//!     
//!     // ... implementa el resto de métodos
//! }
//! ```
//!
//! ¡Y listo! Tu runtime ya es compatible con cualquier parte de Orbit Engine que
//! acepte `impl InputStateExt<K, S>`.
//!
//! ---
//!
//! ## Casos de uso
//!
//! ### 🎮 Gamepad personalizado
//! ```rust,ignore
//! #[derive(Copy, Clone, PartialEq, Eq, Hash)]
//! enum GamepadButton { A, B, X, Y, Start, Select }
//!
//! impl InputStateExt<GamepadButton, ButtonState> for MyGamepad {
//!     // Tu implementación...
//! }
//! ```
//!
//! ### 🕹️ Arcade stick
//! ```rust,ignore
//! #[derive(Copy, Clone, PartialEq, Eq, Hash)]
//! enum ArcadeInput { Up, Down, Left, Right, Button1, Button2 }
//!
//! impl InputStateExt<ArcadeInput, StickState> for ArcadeStick {
//!     // Tu implementación...
//! }
//! ```
//!
//! ### 🖱️ Input combinado (teclado + mouse)
//! ```rust,ignore
//! #[derive(Copy, Clone, PartialEq, Eq, Hash)]
//! enum UnifiedInput {
//!     Key(KeyCode),
//!     Mouse(MouseButton),
//! }
//!
//! impl InputStateExt<UnifiedInput, InputState> for HybridInput {
//!     // Tu implementación...
//! }
//! ```
//!
//! ### 🧪 Testing y mocks
//! ```rust,ignore
//! pub struct MockInput {
//!     pressed_keys: HashSet<KeyCode>,
//! }
//!
//! impl InputStateExt<KeyCode, KeyState> for MockInput {
//!     // Implementación simplificada para tests
//! }
//! ```
//!
//! ---
//!
//! ## Implementación oficial
//!
//! Para la mayoría de casos de uso, la implementación oficial [`orbit_input`](https://crates.io/crates/orbit_input)
//! es suficiente. Incluye:
//!
//! - ✅ Soporte nativo para Linux (evdev) y Windows (winapi)
//! - ✅ Tipos `KeyCode` y `KeyState` predefinidos
//! - ✅ Sistema de historial completo (`WithHistoryExt`)
//! - ✅ Detección de combos, secuencias y doble tap
//! - ✅ Runtime asíncrono con Tokio
//! - ✅ API ergonómica lista para usar
//!
//! ---
//!
//! ## Características
//!
//! - 🔌 **Arquitectura plugin** — cualquier backend puede implementar los traits
//! - 🎯 **Type-safe** — los tipos genéricos previenen errores en tiempo de compilación
//! - 📦 **`no_std` compatible** — puede usarse en sistemas embebidos
//! - 🧩 **Sin dependencias** — solo traits, cero dependencias externas
//! - 🔄 **Versionado semántico estricto** — cambios breaking solo en versiones mayores
//!
//! ---
//!
//! ## Convenciones de tipos genéricos
//!
//! Los traits usan nombres consistentes para sus parámetros genéricos:
//!
//! - **`KeyExt<B, N>`**:
//!   - `B` = **B**ackend (tipo nativo del sistema, ej: `evdev::Key`, `u16`)
//!   - `N` = **N**ormalized (tipo normalizado del runtime, ej: `KeyCode`)
//!
//! - **`KeyStateExt<I, O>`**:
//!   - `I` = **I**nput (estado externo/nativo del sistema)
//!   - `O` = **O**utput (estado interno/normalizado del runtime)
//!
//! - **`InputStateExt<K, S>`**:
//!   - `K` = **K**ey (tipo de tecla, debe ser `Copy + PartialEq + Hash`)
//!   - `S` = **S**tate (tipo de estado, debe ser `Copy + PartialEq`)
//!
//! - **`WithHistoryExt<K, S, T>`**:
//!   - `K` = **K**ey (mismo que arriba)
//!   - `S` = **S**tate (mismo que arriba)
//!   - `T` = **T**ype (tipo de evento, debe implementar `InputEvent`)
//!
//! ---
//!
//! ## Roadmap
//!
//! Futuras versiones del protocolo incluirán:
//!
//! - 🎮 Traits para otros dispositivos (mouse, gamepad, touch)
//! - 📝 Trait para interpretación de texto y layouts de teclado
//! - 🔊 Trait para feedback háptico
//! - 🎯 Trait para gestión de contextos de input (menú, gameplay, diálogo)
//!
//! ---
//!
//! ## Módulos
//!
//! - [`traits`]: Todos los traits disponibles para implementación


pub mod traits;

// Re-exports limpios
pub use traits::*;



pub use traits::keys::{KeyExt, KeyStateExt};
pub use traits::runtime::{RuntimeExt};
pub use traits::state::{InputEvent, InputStateExt, WithHistoryExt};