use std::time::{Duration, Instant};
use std::hash::Hash;

/// # Trait `InputEvent`
///
/// Define la estructura base de un **evento de entrada histórico**.
///
/// Este trait representa un evento individual registrado en el historial,
/// conteniendo la información mínima necesaria para análisis temporal:
/// la tecla involucrada, su estado, y el momento en que ocurrió.
///
/// ## Propósito
/// - Estandarizar la representación de eventos históricos.
/// - Permitir que los backends definan sus propios tipos de evento.
/// - Facilitar el análisis temporal y detección de patrones.
///
/// ## Ejemplo de implementación
/// ```rust,ignore
/// use std::time::Instant;
/// use orbit_input_core::traits::{InputEvent};
/// use orbit_input_core::keyboard::{KeyCode, KeyState};
///
/// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// pub struct KeyEvent {
///     pub keycode: KeyCode,
///     pub state: KeyState,
///     pub timestamp: Instant,
/// }
///
/// impl InputEvent for KeyEvent {
///     type Key = KeyCode;
///     type State = KeyState;
///     
///     fn key(&self) -> Self::Key {
///         self.keycode
///     }
///     
///     fn state(&self) -> Self::State {
///         self.state
///     }
///     
///     fn timestamp(&self) -> Instant {
///         self.timestamp
///     }
/// }
/// ```
pub trait InputEvent: Hash + PartialEq + Clone {
    /// Tipo de tecla usado por este evento.
    type Key: Copy + PartialEq + Hash;
    
    /// Tipo de estado usado por este evento.
    type State: Copy + PartialEq;
    
    /// Retorna la tecla asociada a este evento.
    fn key(&self) -> Self::Key;
    
    /// Retorna el estado de la tecla en este evento.
    fn state(&self) -> Self::State;
    
    /// Retorna el instante temporal en que ocurrió este evento.
    fn timestamp(&self) -> Instant;
}

/// # Trait `InputStateExt`
///
/// Define la interfaz base para la **gestión del estado actual del sistema de entrada**.
///
/// Este trait representa el comportamiento del *estado vivo* del teclado en un frame determinado,
/// permitiendo verificar qué teclas están activas, recién presionadas, soltadas, o cuánto tiempo llevan activas.
///
/// Es la capa fundamental del sistema de input del motor — usada directamente por los *runtimes*
/// (`orbit_input_linux`, `orbit_input_windows`, etc.) y por la capa superior ([`WithHistoryExt`])
/// que añade soporte de historial temporal.
///
/// ## Propósito
/// - Ofrecer una vista directa y eficiente del estado de las teclas.
/// - Permitir consultas rápidas sobre la actividad del input.
/// - Ser agnóstico del backend: cada plataforma puede implementar su propia versión.
///
/// ## Parámetros genéricos
/// - `K`: Tipo de tecla (por ejemplo, [`KeyCode`](crate::keyboard::KeyCode)).
/// - `S`: Tipo de estado (por ejemplo, [`KeyState`](crate::keyboard::KeyState)).
///
/// ## Ejemplo de uso
/// ```rust,ignore
/// use orbit_input_core::keyboard::{KeyCode, KeyState};
/// use orbit_input_core::traits::InputStateExt;
///
/// fn check_player_jump<I: InputStateExt<KeyCode, KeyState>>(input: &I) {
///     if input.is_just_press(KeyCode::Space) {
///         println!("El jugador saltó!");
///     }
///     
///     if input.active_combo(&[KeyCode::ControlLeft, KeyCode::S]) {
///         println!("Guardando partida...");
///     }
/// }
/// ```
pub trait InputStateExt<K, S>
where
    K: Copy + PartialEq + Hash,
    S: Copy + PartialEq,
{
    /// Establece o actualiza el estado de una tecla.
    ///
    /// Normalmente llamado por el runtime cuando detecta un evento de entrada.
    fn set_key(&mut self, key: K, state: S);

    /// Retorna `true` si la tecla fue presionada **por primera vez** en este frame.
    ///
    /// Se diferencia de [`is_pressed`](Self::is_pressed) en que solo retorna `true`
    /// en el frame exacto donde ocurrió la pulsación, no mientras se mantiene presionada.
    fn is_just_press(&self, key: K) -> bool;

    /// Retorna `true` si la tecla está actualmente **presionada o mantenida**.
    ///
    /// Retorna `true` tanto en el frame inicial como en todos los frames subsiguientes
    /// mientras la tecla siga presionada.
    fn is_pressed(&self, key: K) -> bool;

    /// Retorna `true` si la tecla está **completamente liberada**.
    ///
    /// Es el estado opuesto a [`is_pressed`](Self::is_pressed).
    fn is_released(&self, key: K) -> bool;

    /// Retorna `true` si la tecla fue **liberada en este frame específico**.
    ///
    /// Similar a [`is_just_press`](Self::is_just_press) pero para el evento de liberación.
    fn is_just_released(&self, key: K) -> bool;

    /// Retorna el tiempo total que una tecla ha estado presionada.
    ///
    /// Útil para detectar pulsaciones largas (hold) o cargar acciones.
    fn time_pressed(&self, key: K) -> Option<Duration>;

    /// Verifica si una combinación de teclas se encuentra activa (todas presionadas).
    ///
    /// Útil para detectar combinaciones como `CTRL + S` o `SHIFT + A`.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// if input.active_combo(&[KeyCode::ControlLeft, KeyCode::C]) {
    ///     copy_to_clipboard();
    /// }
    /// ```
    fn active_combo(&self, combo: &[K]) -> bool;

    /// Retorna `true` si **cualquier tecla** se encuentra actualmente presionada.
    ///
    /// Útil para detectar actividad general del usuario.
    fn any_pressed(&self) -> bool;

    /// Devuelve la última tecla presionada (si existe).
    ///
    /// Útil para sistemas de rebinding de teclas o debug.
    fn last_pressed(&self) -> Option<K>;

    /// Retorna todas las teclas actualmente presionadas.
    ///
    /// Útil para visualizar el estado completo o debug.
    fn keys_pressed(&self) -> Vec<K>;

    /// Resetea el estado actual (por ejemplo, al cambiar de escena o al pausar el juego).
    ///
    /// Limpia todos los estados internos sin afectar el historial (si existe).
    fn reset(&mut self);


}

/// # Trait `WithHistoryExt`
///
/// Extiende [`InputStateExt`] añadiendo un **sistema de historial de eventos**.
///
/// Este trait permite registrar, consultar y analizar los eventos de entrada
/// a lo largo del tiempo — ideal para detectar *combos*, *doble taps*, *secuencias*
/// o simplemente para fines de depuración o replays.
///
/// ## Concepto
/// - Mientras [`InputStateExt`] maneja el estado **presente**,
///   `WithHistoryExt` maneja el **pasado reciente**.
/// - Cada evento registrado (`T`) representa una acción individual del usuario
///   con su respectivo timestamp, permitiendo análisis temporal sofisticado.
///
/// ## Parámetros genéricos
/// - `K`: Tipo de tecla o código de entrada (por ejemplo, `KeyCode`).
/// - `S`: Estado asociado a la tecla (`KeyState`).
/// - `T`: Tipo de evento histórico que implementa [`InputEvent`].
///
/// ## Ejemplo
/// ```rust,ignore
/// use orbit_input_core::keyboard::{KeyCode, KeyState};
/// use orbit_input_core::traits::WithHistoryExt;
/// use std::time::Duration;
///
/// fn check_combo<H>(history: &H)
/// where
///     H: WithHistoryExt<KeyCode, KeyState, KeyEvent>
/// {
///     // Detectar combo clásico de Konami Code
///     let konami = [
///         KeyCode::ArrowUp, KeyCode::ArrowUp,
///         KeyCode::ArrowDown, KeyCode::ArrowDown,
///         KeyCode::ArrowLeft, KeyCode::ArrowRight,
///     ];
///     
///     if history.match_sequence_in_time(&konami, Duration::from_secs(5)) {
///         println!("¡Konami Code activado!");
///         unlock_secret_level();
///     }
///     
///     // Detectar doble tap para dash
///     if history.is_double_tap(KeyCode::ShiftLeft, Duration::from_millis(300)) {
///         println!("¡Dash activado!");
///     }
/// }
/// ```
pub trait WithHistoryExt<K, S, T>: InputStateExt<K, S>
where
    K: Copy + PartialEq + Hash,
    S: Copy + PartialEq,
    T: InputEvent<Key = K, State = S>,
{
    // === ACCESO BASE ===

    /// Devuelve todos los eventos registrados en el historial.
    ///
    /// Los eventos están ordenados del más antiguo al más reciente.
    fn history(&self) -> &[T];

    /// Devuelve el último evento registrado (más reciente).
    fn last_event(&self) -> Option<&T>;

    /// Limpia por completo el historial de eventos.
    ///
    fn clear_history(&mut self);

    /// Limita el historial a un número máximo de eventos (para controlar el consumo de memoria).
    ///
    /// Elimina los eventos más antiguos si se excede el límite.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // Mantener solo los últimos 100 eventos
    /// history.trim_history(100);
    /// ```
    fn trim_history(&mut self, max: usize);

    // === CONSULTAS TEMPORALES ===

    /// Devuelve el tiempo transcurrido desde el último evento registrado.
    ///
    /// Útil para detectar inactividad del usuario.
    fn since_last_event(&self) -> Duration;

    /// Devuelve el tiempo desde la última vez que se presionó una tecla específica.
    ///
    /// Retorna `None` si la tecla nunca fue presionada.
    fn since_key_pressed(&self, key: K) -> Option<std::time::Duration>;

    /// Devuelve la diferencia temporal entre los dos últimos eventos consecutivos de la misma tecla.
    ///
    /// Útil para medir velocidad de tapping o intervalos de pulsación.
    fn delta_between(&self, key: K) -> Option<std::time::Duration>;

    /// Verifica si una tecla fue presionada dos veces dentro de un intervalo determinado (doble tap).
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // Detectar doble clic en menos de 300ms
    /// if history.is_double_tap(KeyCode::Space, std::time::Duration::from_millis(300)) {
    ///     perform_double_jump();
    /// }
    /// ```
    fn is_double_tap(&self, key: K, threshold: std::time::Duration) -> bool;

    /// Calcula el promedio de tiempo entre pulsaciones consecutivas de una tecla.
    ///
    /// Útil para análisis de ritmo o detección de patrones de entrada.
    fn average_press_interval(&self, key: K) -> Option<std::time::Duration>;

    // === DETECCIÓN DE COMBOS Y SECUENCIAS ===

    /// Verifica si una secuencia específica de teclas ocurrió en el orden indicado.
    ///
    /// No considera el tiempo entre eventos, solo el orden.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // Detectar secuencia clásica: arriba, arriba, abajo, abajo
    /// let pattern = [KeyCode::ArrowUp, KeyCode::ArrowUp, 
    ///                KeyCode::ArrowDown, KeyCode::ArrowDown];
    /// if history.match_sequence(&pattern) {
    ///     println!("Secuencia detectada!");
    /// }
    /// ```
    fn match_sequence(&self, pattern: &[K]) -> bool;

    /// Verifica si una secuencia de teclas ocurrió dentro de un margen temporal determinado.
    ///
    /// Útil para combos rápidos o inputs en cadena que requieren timing preciso.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // La secuencia debe completarse en menos de 2 segundos
    /// if history.match_sequence_in_time(&combo, Duration::from_secs(2)) {
    ///     activate_special_move();
    /// }
    /// ```
    fn match_sequence_in_time(&self, pattern: &[K], window: std::time::Duration) -> bool;

    /// Verifica si un conjunto de teclas fue presionado de forma simultánea dentro de una tolerancia de tiempo.
    ///
    /// Ideal para detectar combinaciones como `CTRL + C` o `SHIFT + ALT + S` donde
    /// las teclas deben presionarse "al mismo tiempo" (dentro de la tolerancia).
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // Detectar CTRL+SHIFT+S con tolerancia de 100ms
    /// let combo = [KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::S];
    /// if history.simultaneous_combo(&combo, Duration::from_millis(100)) {
    ///     save_as();
    /// }
    /// ```
    fn simultaneous_combo(&self, combo: &[K], tolerance: std::time::Duration) -> bool;

    // === FILTRADO Y BÚSQUEDA ===

    /// Devuelve los últimos `n` eventos registrados para una tecla específica.
    ///
    /// Los eventos están ordenados del más antiguo al más reciente.
    fn find_last_n(&self, key: K, n: usize) -> Vec<&T>;

    /// Devuelve todas las teclas presionadas dentro de un rango temporal dado.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // Obtener todas las teclas presionadas en el último segundo
    /// let recent_keys = history.keys_in_last(std::time::Duration::from_secs(1));
    /// ```
    fn keys_in_last(&self, duration: std::time::Duration) -> Vec<K>;

    /// Verifica si una tecla fue presionada recientemente (dentro de los últimos `n` eventos).
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// // ¿La tecla Escape fue presionada en los últimos 5 eventos?
    /// if history.occurred_recently(KeyCode::Escape, 5) {
    ///     show_menu();
    /// }
    /// ```
    fn occurred_recently(&self, key: K, within: usize) -> bool;

    /// Devuelve cuántas veces una tecla fue presionada en los últimos `n` eventos.
    ///
    /// Útil para detectar spam de teclas o medir frecuencia de uso.
    fn count_recent(&self, key: K, within: usize) -> usize;

    // === ESTADÍSTICAS ===

    /// Devuelve el total de veces que una tecla fue presionada desde el inicio del historial.
    fn total_presses(&self, key: K) -> usize;

    /// Calcula la frecuencia promedio de pulsaciones por segundo de una tecla.
    ///
    /// Basado en el historial completo disponible.
    fn press_frequency(&self, key: K) -> f32;

    /// Devuelve la tecla más utilizada dentro del historial.
    ///
    /// Útil para análisis de gameplay o sistemas de tutoriales adaptativos.
    fn most_frequent_key(&self) -> Option<K>;

    /// Calcula la velocidad promedio de entrada (teclas por segundo globales).
    ///
    /// Considera todas las teclas en el historial.
    fn average_input_speed(&self) -> f32;

    // === UTILIDADES AVANZADAS ===

    /// Crea un iterador sobre todos los eventos del historial.
    ///
    /// Útil para sistemas de replay o análisis personalizado.
    ///
    /// # Ejemplo
    /// ```rust,ignore
    /// for event in history.replay() {
    ///     println!("Key: {:?}, State: {:?}, Time: {:?}", 
    ///              event.key(), event.state(), event.timestamp());
    /// }
    /// ```
    fn replay<'a>(&'a self) -> impl Iterator<Item = &'a T> where T: 'a;

    /// Elimina y retorna el último evento del historial.
    ///
    /// Útil para sistemas de undo o rollback.
    fn undo_last(&mut self) -> Option<T>;
}