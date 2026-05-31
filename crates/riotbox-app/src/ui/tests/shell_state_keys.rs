#[test]
fn shell_state_handles_help_refresh_and_action_keys() {
    let mut shell = sample_shell_state();

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);
    assert_eq!(shell.status_message, "help overlay opened");

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('r')),
        ShellKeyOutcome::RequestRefresh
    );
    assert_eq!(shell.status_message, "re-ingest source requested");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('2')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Log);
    assert_eq!(shell.status_message, "switched to log screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('3')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Source);
    assert_eq!(shell.status_message, "switched to source screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('4')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Capture);
    assert_eq!(shell.status_message, "switched to capture screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.status_message,
        "open Jam first if you want to use inspect"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Tab),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Jam);
    assert_eq!(shell.status_message, "switched to jam screen");
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Inspect);
    assert_eq!(
        shell.status_message,
        "opened Jam inspect | press i to return to perform"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(shell.status_message, "returned Jam to perform mode");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('m')),
        ShellKeyOutcome::QueueSceneMutation
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('y')),
        ShellKeyOutcome::QueueSceneSelect
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('Y')),
        ShellKeyOutcome::QueueSceneRestore
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('b')),
        ShellKeyOutcome::QueueMc202RoleToggle
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('g')),
        ShellKeyOutcome::QueueMc202GenerateFollower
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('a')),
        ShellKeyOutcome::QueueMc202GenerateAnswer
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('P')),
        ShellKeyOutcome::QueueMc202GeneratePressure
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('I')),
        ShellKeyOutcome::QueueMc202GenerateInstigator
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('G')),
        ShellKeyOutcome::QueueMc202MutatePhrase
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('f')),
        ShellKeyOutcome::QueueTr909Fill
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('d')),
        ShellKeyOutcome::QueueTr909Reinforce
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('s')),
        ShellKeyOutcome::QueueTr909Slam
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('t')),
        ShellKeyOutcome::QueueTr909Takeover
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('k')),
        ShellKeyOutcome::QueueTr909SceneLock
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('x')),
        ShellKeyOutcome::QueueTr909Release
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('c')),
        ShellKeyOutcome::QueueCaptureBar
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('-')),
        ShellKeyOutcome::PreviousCaptureLength
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('=')),
        ShellKeyOutcome::NextCaptureLength
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('p')),
        ShellKeyOutcome::PromoteLastCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('w')),
        ShellKeyOutcome::QueueW30TriggerPad
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('n')),
        ShellKeyOutcome::QueueW30StepFocus
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('B')),
        ShellKeyOutcome::QueueW30SwapBank
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('j')),
        ShellKeyOutcome::QueueW30BrowseSlicePool
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('D')),
        ShellKeyOutcome::QueueW30ApplyDamageProfile
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('z')),
        ShellKeyOutcome::QueueW30LoopFreeze
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('l')),
        ShellKeyOutcome::QueueW30LiveRecall
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('o')),
        ShellKeyOutcome::QueueW30Audition
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('e')),
        ShellKeyOutcome::QueueW30Resample
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('E')),
        ShellKeyOutcome::QueueProductMixExport
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('C')),
        ShellKeyOutcome::ConfirmSourceTimingGrid
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('R')),
        ShellKeyOutcome::RevertSourceTimingGrid
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Left),
        ShellKeyOutcome::NavigateSourceMapPreviousBar
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Right),
        ShellKeyOutcome::NavigateSourceMapNextBar
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Up),
        ShellKeyOutcome::NavigateSourceMapPreviousPhrase
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Down),
        ShellKeyOutcome::NavigateSourceMapNextPhrase
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('v')),
        ShellKeyOutcome::TogglePinLatestCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('[')),
        ShellKeyOutcome::LowerDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(']')),
        ShellKeyOutcome::RaiseDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('<')),
        ShellKeyOutcome::LowerMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('>')),
        ShellKeyOutcome::RaiseMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Enter),
        ShellKeyOutcome::AcceptCurrentGhostSuggestion
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('N')),
        ShellKeyOutcome::RejectCurrentGhostSuggestion
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('u')),
        ShellKeyOutcome::UndoLast
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(' ')),
        ShellKeyOutcome::ToggleTransport
    );

    assert_eq!(shell.handle_key_code(KeyCode::Esc), ShellKeyOutcome::Quit);
}

#[test]
fn first_run_shell_blocks_jam_inspect_toggle() {
    let mut shell = first_run_shell_state();

    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.status_message,
        "finish the first guided move before opening inspect"
    );
}
