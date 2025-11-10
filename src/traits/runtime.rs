/// # Trait `RuntimeExt`
///
/// Define el **ciclo de vida y gestión** de un runtime de entrada completo.
///
/// Este trait representa la capa más alta de abstracción del sistema de input:
/// un **runtime asíncrono** que mantiene un loop continuo para:
///
/// 1. **Escuchar** eventos del backend nativo (evdev, Win32, HID, etc.)
/// 2. **Traducir** esos eventos usando [`KeyExt`](crate::KeyExt) y [`KeyStateExt`](crate::KeyStateExt)
/// 3. **Actualizar** el estado compartido que implementa [`InputStateExt`](crate::InputStateExt)
///
/// ## Responsabilidades
///
/// - ✅ Gestionar el ciclo de vida del sistema de input (init, run, stop)
/// - ✅ Mantener un loop asíncrono de captura de eventos
/// - ✅ Actualizar automáticamente el estado compartido
/// - ✅ Proveer información de monitoreo y diagnóstico
/// - ❌ **NO** maneja el game loop principal (eso es responsabilidad del motor)
///
/// ## Diagrama conceptual
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                      Input Runtime                           │
/// │                                                              │
/// │  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐    │
/// │  │ OS Backend   │ → │ KeyExt +     │ → │ InputState   │    │
/// │  │ (evdev/Win32)│   │ KeyStateExt  │   │ (compartido) │    │
/// │  └──────────────┘   └──────────────┘   └──────────────┘    │
/// │                                                              │
/// │  async loop {                                                │
/// │      let event = poll_system();                              │
/// │      let (key, state) = translate(event);                    │
/// │      shared_state.lock().set_key(key, state);                │
/// │  }                                                           │
/// └──────────────────────────────────────────────────────────────┘
///                             ▲
///                             │
///                    Game Loop consulta estado
/// ```
///
/// ## Parámetros genéricos
///
/// - `S`: Tipo del estado compartido (debe implementar [`InputStateExt`](crate::InputStateExt))
///
/// ## Consideraciones de diseño
///
/// ### Sincronización
/// Cada implementación debe decidir cómo sincronizar el estado compartido:
/// - `Arc<Mutex<S>>` (std)
/// - `Arc<RwLock<S>>` (std o parking_lot)
/// - `Arc<AtomicRefCell<S>>` (single-threaded)
/// - Channels/message passing
///
/// El trait **no impone** un mecanismo específico — cada runtime elige el suyo.
///
/// ### Modelo asíncrono
/// Los métodos `initialize` y `run` son `async` para permitir:
/// - I/O no bloqueante con el sistema operativo
/// - Uso de runtimes async (Tokio, async-std, smol)
/// - Integración con otros sistemas asíncronos del motor
///
/// ## Ejemplo de implementación (conceptual)
///
/// ```rust,ignore
/// use orbit_input_core::{RuntimeExt, InputStateExt, KeyExt, KeyStateExt};
/// use std::sync::{Arc, Mutex};
///
/// pub struct MyRuntime {
///     running: bool,
///     events_count: usize,
///     state: Arc<Mutex<MyInputState>>,
/// }
///
/// impl RuntimeExt for MyRuntime {
///     type Error = std::io::Error;
///     type State = MyInputState;
///     type SharedState = Arc<Mutex<MyInputState>>;
///     
///     fn new() -> Result<(Self, Self::SharedState), Self::Error> {
///         let state = Arc::new(Mutex::new(MyInputState::new()));
///         let runtime = Self {
///             running: false,
///             events_count: 0,
///             state: state.clone(),
///         };
///         Ok((runtime, state))
///     }
///     
///     async fn initialize(&mut self) -> Result<(), Self::Error> {
///         // Abrir dispositivos, configurar recursos...
///         self.running = true;
///         Ok(())
///     }
///     
///     async fn run(&mut self) -> Result<(), Self::Error> {
///         while self.running {
///             // Leer evento del OS
///             let native_event = read_system_event().await?;
///             
///             // Traducir usando KeyExt
///             let key = MyKeyCode::from_backend_key(native_event.key);
///             let state = MyKeyState::from_external_state(native_event.state);
///             
///             // Actualizar estado compartido
///             self.state.lock().unwrap().set_key(key, state);
///             self.events_count += 1;
///         }
///         Ok(())
///     }
///     
///     fn stop(&mut self) -> Result<(), Self::Error> {
///         self.running = false;
///         Ok(())
///     }
///     
///     // ... resto de métodos
/// }
/// ```
///
/// ## Uso desde el game loop
///
/// ```rust,ignore
/// // Inicializar runtime
/// let (mut runtime, input_state) = MyRuntime::new()?;
///
/// // Spawnearlo en background
/// tokio::spawn(async move {
///     runtime.initialize().await.unwrap();
///     runtime.run().await.unwrap();
/// });
///
/// // En el game loop:
/// loop {
///     let state = input_state.lock().unwrap();
///     
///     if state.is_just_press(MyKeyCode::Escape) {
///         break;
///     }
///     
///     if state.is_pressed(MyKeyCode::W) {
///         player.move_forward();
///     }
/// }
/// ```
///
/// ## Relación con otros traits
///
/// - [`InputStateExt`](crate::InputStateExt): El estado que este runtime actualiza
/// - [`KeyExt`](crate::KeyExt): Usado para traducir teclas del backend
/// - [`KeyStateExt`](crate::KeyStateExt): Usado para traducir estados del backend
/// - [`WithHistoryExt`](crate::WithHistoryExt): Opcionalmente el estado puede incluir historial
pub trait RuntimeExt {
    /// Tipo de error retornado por el runtime (específico del backend).
    ///
    /// Ejemplos:
    /// - `std::io::Error` para backends basados en I/O
    /// - `evdev::Error` para Linux
    /// - Custom error enums para casos específicos
    type Error;

    /// Tipo del estado interno que mantiene el runtime.
    ///
    /// Debe implementar [`InputStateExt`](crate::InputStateExt) y ser thread-safe
    /// (`Send + Sync + 'static`).
    type State: Send + Sync + 'static;

    /// Tipo del contenedor compartido para el estado.
    ///
    /// Cada implementación decide su mecanismo de sincronización:
    /// - `Arc<Mutex<Self::State>>`
    /// - `Arc<RwLock<Self::State>>`
    /// - `Arc<parking_lot::RwLock<Self::State>>`
    /// - Canales de mensajes, etc.
    type SharedState: Clone + Send + Sync + 'static;

    // ==================== INICIALIZACIÓN Y CONTROL ====================

    /// Crea una nueva instancia del runtime y su estado compartido.
    ///
    /// Esta función debe:
    /// 1. Crear el estado interno (vacío/por defecto)
    /// 2. Envolverlo en el mecanismo de sincronización elegido
    /// 3. Retornar tanto el runtime como una referencia al estado compartido
    ///
    /// # Errores
    ///
    /// Puede fallar si:
    /// - No se pueden detectar dispositivos de entrada
    /// - Faltan permisos del sistema
    /// - El backend no está disponible en esta plataforma
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// let (runtime, state) = MyRuntime::new()?;
    /// // `state` puede compartirse con el game loop
    /// // `runtime` se spawneará en un task asíncrono
    /// ```
    fn new() -> Result<(Self, Self::SharedState), Self::Error>
    where
        Self: Sized;

    /// Inicializa los recursos necesarios antes de comenzar la captura de eventos.
    ///
    /// Este método debe ser llamado **antes** de `run()` y puede:
    /// - Abrir dispositivos de entrada
    /// - Configurar permisos
    /// - Preparar buffers o estructuras internas
    /// - Registrar callbacks del sistema
    ///
    /// # Errores
    ///
    /// Retorna error si la inicialización falla (por ejemplo, dispositivo no disponible).
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// runtime.initialize().await?;
    /// ```
    fn initialize(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Inicia el **loop principal** de captura y procesamiento de eventos.
    ///
    /// Este método debe:
    /// 1. Entrar en un loop continuo (hasta que `stop()` sea llamado)
    /// 2. Leer eventos del backend nativo
    /// 3. Traducirlos usando los traits de conversión
    /// 4. Actualizar el estado compartido
    ///
    /// **Este método bloquea** hasta que el runtime sea detenido.
    ///
    /// # Errores
    ///
    /// Puede retornar error si:
    /// - El dispositivo se desconecta
    /// - Ocurre un error crítico de I/O
    /// - El sistema deniega acceso
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// // Típicamente spawneado en un task:
    /// tokio::spawn(async move {
    ///     runtime.run().await.expect("Runtime falló");
    /// });
    /// ```
    fn run(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Detiene la captura de eventos y libera los recursos del runtime.
    ///
    /// Después de llamar este método:
    /// - El loop de `run()` debe terminar
    /// - Los dispositivos deben cerrarse
    /// - Los recursos deben liberarse
    ///
    /// El runtime puede ser reiniciado con `restart()`.
    ///
    /// # Errores
    ///
    /// Puede fallar si hay problemas cerrando recursos del sistema.
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// Reinicia la captura de eventos.
    ///
    /// Útil cuando:
    /// - Un dispositivo se reconecta
    /// - Se necesita recargar la configuración
    /// - Hubo un error recuperable
    ///
    /// Internamente puede llamar a `stop()` seguido de `initialize()` y `run()`.
    ///
    /// # Errores
    ///
    /// Puede fallar si el reinicio no es posible (mismo tipo de errores que `new()`).
    fn restart(&mut self) -> Result<(), Self::Error>;

    // ==================== MONITOREO Y ESTADO ====================

    /// Retorna `true` si el runtime se encuentra activo (capturando eventos).
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// if runtime.is_running() {
    ///     println!("Capturando input...");
    /// }
    /// ```
    fn is_running(&self) -> bool;

    /// Retorna la cantidad de eventos procesados desde que se inició el runtime.
    ///
    /// Útil para:
    /// - Debugging
    /// - Estadísticas de rendimiento
    /// - Detección de problemas (ej: contador no aumenta = runtime congelado)
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// println!("Eventos procesados: {}", runtime.events_processed());
    /// ```
    fn events_processed(&self) -> usize;

    /// Retorna una descripción textual del backend o estado actual.
    ///
    /// Útil para:
    /// - Logging
    /// - UI de debug
    /// - Diagnóstico de errores
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// println!("Backend: {}", runtime.backend_name());
    /// // Output: "Linux (evdev)" o "Windows (Raw Input)"
    /// ```
    fn backend_name(&self) -> &'static str;

    /// Reinicia el estado de entrada **sin detener el runtime**.
    ///
    /// Limpia todas las teclas marcadas como presionadas, historial, etc.
    ///
    /// Útil cuando:
    /// - Se cambia de escena/nivel
    /// - Se pausa el juego
    /// - Se quiere ignorar input acumulado
    ///
    /// **Nota:** El runtime continúa capturando eventos, solo se resetea el estado.
    ///
    /// # Ejemplo
    ///
    /// ```rust,ignore
    /// // Al cambiar de escena:
    /// runtime.reset_state();
    /// load_new_level();
    /// ```
    fn reset_state(&mut self);
}