pub(in crate::ui) const GESTURE_MUTATE: &str = "mutate";

pub(in crate::ui) const GESTURE_SCENE_JUMP: &str = "scene jump";

pub(in crate::ui) const GESTURE_RESTORE: &str = "restore";

pub(in crate::ui) const GESTURE_VOICE: &str = "voice";

pub(in crate::ui) const GESTURE_FOLLOW: &str = "follow";

pub(in crate::ui) const GESTURE_ANSWER: &str = "answer";

pub(in crate::ui) const GESTURE_PRESSURE: &str = "pressure";

pub(in crate::ui) const GESTURE_INSTIGATE: &str = "instigate";

pub(in crate::ui) const GESTURE_PHRASE: &str = "phrase";

pub(in crate::ui) const GESTURE_FILL: &str = "fill";

pub(in crate::ui) const GESTURE_PUSH: &str = "push";

pub(in crate::ui) const GESTURE_SLAM: &str = "slam";

pub(in crate::ui) const GESTURE_TAKEOVER: &str = "takeover";

pub(in crate::ui) const GESTURE_LOCK: &str = "lock";

pub(in crate::ui) const GESTURE_RELEASE: &str = "release";

pub(in crate::ui) const GESTURE_CAPTURE: &str = "capture";

pub(in crate::ui) const GESTURE_PROMOTE: &str = "promote";

pub(in crate::ui) const GESTURE_HIT: &str = "hit";

pub(in crate::ui) const GESTURE_NEXT_PAD: &str = "next pad";

pub(in crate::ui) const GESTURE_BANK: &str = "bank";

pub(in crate::ui) const GESTURE_BROWSE: &str = "browse";

pub(in crate::ui) const GESTURE_DAMAGE: &str = "damage";

pub(in crate::ui) const GESTURE_TURNAROUND: &str = "turnaround";

pub(in crate::ui) const GESTURE_PITCH_DIVE: &str = "pitch dive";

pub(in crate::ui) const GESTURE_FILTER_SLAM: &str = "filter slam";

pub(in crate::ui) const GESTURE_FREEZE: &str = "freeze";

pub(in crate::ui) const GESTURE_RECALL: &str = "recall";

pub(in crate::ui) const GESTURE_AUDITION: &str = "audition";

pub(in crate::ui) const GESTURE_RESAMPLE: &str = "resample";

pub(in crate::ui) const GESTURE_EXPORT: &str = "export";

pub(in crate::ui) const GESTURE_TOUCH: &str = "touch";

pub(in crate::ui) const GESTURE_UNDO: &str = "undo";

pub(in crate::ui) const ADVANCED_GESTURES: &[(&str, &str)] = &[
    ("Y", GESTURE_RESTORE),
    ("g", GESTURE_FOLLOW),
    ("a", GESTURE_ANSWER),
    ("b", GESTURE_VOICE),
    ("P", GESTURE_PRESSURE),
    ("I", GESTURE_INSTIGATE),
    ("G", GESTURE_PHRASE),
    ("d", GESTURE_PUSH),
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
];

pub(in crate::ui) const LANE_GESTURES: &[(&str, &str)] = &[
    ("< >", GESTURE_TOUCH),
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("z", GESTURE_FREEZE),
    ("e", GESTURE_RESAMPLE),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
    ("H", GESTURE_TURNAROUND),
    ("V", GESTURE_PITCH_DIVE),
    ("L", GESTURE_FILTER_SLAM),
];

pub(in crate::ui) const HELP_PRIMARY_CONFIRM_GESTURES: &[(&str, &str)] =
    &[("c", GESTURE_CAPTURE), ("u", GESTURE_UNDO)];

pub(in crate::ui) const HELP_ADVANCED_GESTURES_A: &[(&str, &str)] = &[
    ("g", GESTURE_FOLLOW),
    ("a", GESTURE_ANSWER),
    ("m", GESTURE_MUTATE),
    ("b", GESTURE_VOICE),
    ("P", GESTURE_PRESSURE),
    ("I", GESTURE_INSTIGATE),
    ("G", GESTURE_PHRASE),
    ("d", GESTURE_PUSH),
];

pub(in crate::ui) const HELP_ADVANCED_GESTURES_B: &[(&str, &str)] = &[
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
    ("x", GESTURE_RELEASE),
];

pub(in crate::ui) const HELP_ADVANCED_GESTURES_C: &[(&str, &str)] = &[
    ("p", GESTURE_PROMOTE),
    ("n", GESTURE_NEXT_PAD),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
];

pub(in crate::ui) const HELP_ADVANCED_GESTURES_D: &[(&str, &str)] = &[
    ("E", GESTURE_EXPORT),
    ("D", GESTURE_DAMAGE),
    ("H", GESTURE_TURNAROUND),
    ("z", GESTURE_FREEZE),
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("e", GESTURE_RESAMPLE),
];

pub(in crate::ui) fn render_gesture_items(items: &[(&str, &str)], separator: &str) -> String {
    items
        .iter()
        .map(|(key, label)| format!("{key}{separator}{label}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

pub(in crate::ui) fn queued_status_message(label: &str, boundary: &str) -> String {
    format!("queue {label} on {boundary}")
}

pub(in crate::ui) fn jam_action_label(command: &str) -> String {
    match command {
        "mutate.scene" => GESTURE_MUTATE.into(),
        "scene.launch" => GESTURE_SCENE_JUMP.into(),
        "scene.restore" => GESTURE_RESTORE.into(),
        "mc202.set_role" => GESTURE_VOICE.into(),
        "mc202.generate_follower" => GESTURE_FOLLOW.into(),
        "mc202.generate_answer" => GESTURE_ANSWER.into(),
        "mc202.generate_pressure" => GESTURE_PRESSURE.into(),
        "mc202.generate_instigator" => GESTURE_INSTIGATE.into(),
        "mc202.mutate_phrase" => GESTURE_PHRASE.into(),
        "tr909.fill_next" => GESTURE_FILL.into(),
        "tr909.reinforce_break" => GESTURE_PUSH.into(),
        "tr909.set_slam" => GESTURE_SLAM.into(),
        "tr909.takeover" => GESTURE_TAKEOVER.into(),
        "tr909.scene_lock" => GESTURE_LOCK.into(),
        "tr909.release" => GESTURE_RELEASE.into(),
        "capture.now" | "capture.loop" | "capture.bar_group" => GESTURE_CAPTURE.into(),
        "promote.capture_to_pad" | "promote.capture_to_scene" => GESTURE_PROMOTE.into(),
        "w30.trigger_pad" => GESTURE_HIT.into(),
        "w30.step_focus" => GESTURE_NEXT_PAD.into(),
        "w30.swap_bank" => GESTURE_BANK.into(),
        "w30.browse_slice_pool" => GESTURE_BROWSE.into(),
        "w30.apply_damage_profile" => GESTURE_DAMAGE.into(),
        "w30.hook_turnaround" => GESTURE_TURNAROUND.into(),
        "w30.pitch_dive" => GESTURE_PITCH_DIVE.into(),
        "w30.filter_slam" => GESTURE_FILTER_SLAM.into(),
        "w30.loop_freeze" => GESTURE_FREEZE.into(),
        "w30.live_recall" => GESTURE_RECALL.into(),
        "w30.audition_raw_capture" => GESTURE_AUDITION.into(),
        "w30.audition_promoted" => GESTURE_AUDITION.into(),
        "promote.resample" => GESTURE_RESAMPLE.into(),
        _ => command.to_string(),
    }
}
