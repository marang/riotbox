use std::{io, path::Path};

use crossterm::event::KeyCode;
use riotbox_core::action::CommitBoundary;

use super::{NdjsonWriter, apply_probe_key, commit_boundary, probe_shell, record_probe_start};

pub(super) fn write_recipe2_mc202_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("recipe2-mc202-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "recipe2-mc202",
        "synthetic-recipe2-mc202-probe.wav",
        "headless-recipe2-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 300, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 400, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('a'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Phrase, 2, 1)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('P'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Phrase, 3, 1)?;
    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('I'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Phrase, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('G'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_300, KeyCode::Char('>'))?;

    Ok(())
}

pub(super) fn write_first_playable_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("first-playable-jam-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "first-playable-jam",
        "synthetic-first-playable-source.wav",
        "headless-first-playable-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;

    Ok(())
}

pub(super) fn write_stage_style_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("stage-style-jam-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "stage-style-jam",
        "synthetic-stage-style-source.wav",
        "headless-stage-style-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;
    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Bar, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;

    Ok(())
}

pub(super) fn write_stage_style_restore_diversity_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("stage-style-restore-diversity-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "stage-style-restore-diversity",
        "synthetic-stage-style-restore-diversity-source.wav",
        "headless-stage-style-restore-diversity-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('c'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Phrase, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('o'))?;
    apply_probe_key(&mut shell, &mut writer, 500, KeyCode::Char('p'))?;
    commit_boundary(&mut shell, &mut writer, 600, CommitBoundary::Bar, 2, 2)?;
    apply_probe_key(&mut shell, &mut writer, 700, KeyCode::Char('w'))?;
    commit_boundary(&mut shell, &mut writer, 800, CommitBoundary::Beat, 3, 1)?;

    apply_probe_key(&mut shell, &mut writer, 900, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 1_000, CommitBoundary::Bar, 4, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('d'))?;
    commit_boundary(&mut shell, &mut writer, 1_200, CommitBoundary::Phrase, 5, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_300, KeyCode::Char('k'))?;
    commit_boundary(&mut shell, &mut writer, 1_400, CommitBoundary::Phrase, 6, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_500, KeyCode::Char('x'))?;
    commit_boundary(&mut shell, &mut writer, 1_600, CommitBoundary::Phrase, 7, 1)?;

    apply_probe_key(&mut shell, &mut writer, 1_700, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 1_800, CommitBoundary::Phrase, 8, 1)?;
    apply_probe_key(&mut shell, &mut writer, 1_900, KeyCode::Char('a'))?;
    commit_boundary(&mut shell, &mut writer, 2_000, CommitBoundary::Phrase, 9, 1)?;
    apply_probe_key(&mut shell, &mut writer, 2_100, KeyCode::Char('P'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_200,
        CommitBoundary::Phrase,
        10,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_300, KeyCode::Char('I'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_400,
        CommitBoundary::Phrase,
        11,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_500, KeyCode::Char('G'))?;
    commit_boundary(
        &mut shell,
        &mut writer,
        2_600,
        CommitBoundary::Phrase,
        12,
        1,
    )?;
    apply_probe_key(&mut shell, &mut writer, 2_700, KeyCode::Char('>'))?;

    Ok(())
}

pub(super) fn write_feral_grid_jam_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = probe_shell("feral-grid-jam-probe");

    record_probe_start(
        &mut writer,
        &mut shell,
        path,
        "feral-grid-jam",
        "synthetic-feral-grid-source.wav",
        "headless-feral-grid-session.json",
    )?;

    apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_probe_key(&mut shell, &mut writer, 200, KeyCode::Char('f'))?;
    commit_boundary(&mut shell, &mut writer, 300, CommitBoundary::Bar, 1, 1)?;
    apply_probe_key(&mut shell, &mut writer, 400, KeyCode::Char('g'))?;
    commit_boundary(&mut shell, &mut writer, 500, CommitBoundary::Phrase, 2, 1)?;
    Ok(())
}
