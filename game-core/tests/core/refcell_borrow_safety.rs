/// Documenta il bug: AudioManager usava borrow_mut() attraverso un .await,
/// causando panic "already borrowed" quando play_sfx o preload_sfx venivano
/// chiamati concorrentemente durante play_cry.
///
/// Il pattern corretto: estrarre tutti i dati necessari PRIMA dell'await,
/// rilasciare il borrow, eseguire l'operazione async, poi riacquisire solo
/// se necessario per scrivere il risultato.
///
/// Questo test verifica che RefCell con borrow tenuto attraverso await
/// causa panic — e che il pattern corretto lo evita.

use std::cell::RefCell;
use std::rc::Rc;

/// Simula il pattern SBAGLIATO: borrow_mut tenuto attraverso un'operazione
/// che potrebbe interrompersi (equivalente async in contesto sincrono).
///
/// Il bug: se un'altra closure tenta borrow_mut mentre il primo è attivo → panic.
#[test]
#[should_panic(expected = "already borrowed")]
fn borrow_mut_sovrapposto_causa_panic() {
    let cell: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
    let cell2 = cell.clone();

    // Prima closure: tiene borrow_mut aperto
    let mut guard = cell.borrow_mut();
    *guard += 1;

    // Seconda closure tenta borrow_mut mentre il primo è ancora attivo → panic
    let _guard2 = cell2.borrow_mut(); // should_panic qui
}

/// Il pattern CORRETTO: rilascia il borrow prima di cedere il controllo,
/// poi riacquista solo per scrivere.
#[test]
fn borrow_rilasciato_prima_di_operazione_concorrente() {
    let cell: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(vec![]));

    // Step 1: leggi ciò che serve, rilascia immediatamente
    let value_to_process = {
        let guard = cell.borrow();
        guard.len() as u32 // copia il dato, drop del guard
    }; // guard rilasciato qui

    // Step 2: operazione "async" (simulata) — nessun borrow attivo
    let result = value_to_process + 42;

    // Step 3: riacquista solo per scrivere
    {
        let mut guard = cell.borrow_mut();
        guard.push(result);
    } // guard rilasciato

    // Step 4: un'altra operazione può ora accedere liberamente
    {
        let mut guard = cell.borrow_mut();
        guard.push(99);
    }

    assert_eq!(*cell.borrow(), vec![42, 99]);
}

/// Verifica che il pattern estratto da AudioManager::play_cry (corretto)
/// non tenga il borrow attraverso l'operazione costosa.
///
/// Pattern corretto:
///   1. borrow() → estrai ctx_ref → drop borrow
///   2. operazione async con ctx_ref (nessun borrow attivo)
///   3. borrow_mut() → scrivi risultato → drop borrow
#[test]
fn pattern_corretto_tre_fasi_senza_sovrapposizione() {
    let state: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(Some("ctx".to_string())));

    // Fase 1: estrai il dato necessario, rilascia subito
    let ctx_data = {
        let guard = state.borrow();
        guard.as_ref().map(|s| s.clone())
    }; // borrow rilasciato

    // Fase 2: "operazione async" — nessun borrow attivo su `state`
    // In questo momento, play_sfx o preload_sfx possono accedere liberamente
    let result = ctx_data.map(|s| format!("processed:{}", s));

    // Verifica che in questo punto un'altra closure possa accedere
    {
        let _guard = state.borrow_mut(); // non deve panicare
    }

    // Fase 3: scrivi il risultato
    {
        let mut guard = state.borrow_mut();
        *guard = result;
    }

    assert_eq!(*state.borrow(), Some("processed:ctx".to_string()));
}
