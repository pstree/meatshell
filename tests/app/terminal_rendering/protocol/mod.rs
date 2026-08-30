use super::*;

#[test]
fn paste_tracks_remote_bracketed_paste_state() {
    let bufs = TermBuffers::default();
    let mut buffer = make_buf(2, 20, &[], &[], 0);
    buffer.parser.process(b"\x1b[?2004h");
    bufs.lock()
        .unwrap()
        .insert("tab".into(), Arc::new(Mutex::new(buffer)));

    assert!(terminal_uses_bracketed_paste(&bufs, "tab"));
    assert!(!terminal_uses_bracketed_paste(&bufs, "missing"));

    let buffer = term_buf(&bufs, "tab").unwrap();
    buffer.lock().unwrap().parser.process(b"\x1b[?2004l");
    assert!(!terminal_uses_bracketed_paste(&bufs, "tab"));
}

#[test]
fn bash_readline_history_repaints_the_current_line() {
    let mut buffer = make_buf(4, 40, &[], &[], 0);
    let _ = buffer.ingest(b"\x1b[?2004hP> echo second");
    // GNU readline replaces "second" with the shorter "first" using six
    // backspaces, DCH for the leftover cell, then the replacement suffix.
    let _ = buffer.ingest(b"\x08\x08\x08\x08\x08\x08\x1b[1Pfirst");
    buffer.render();

    assert_eq!(buffer.displayed_text[0], "P> echo first");
    assert_eq!(buffer.parser.screen().cursor_position(), (0, 13));
}

#[test]
fn terminal_queries_reply_at_the_current_cursor_position() {
    let mut buffer = make_buf(4, 40, &[], &[], 0);

    assert_eq!(buffer.ingest(b"abc\x1b[6n"), b"\x1b[1;4R");
    assert_eq!(buffer.ingest(b"\x1b[?6n"), b"\x1b[?1;4R");
    assert_eq!(
        buffer.ingest(b"\x1b[5n\x1b[c\x1b[0c"),
        b"\x1b[0n\x1b[?1;2c\x1b[?1;2c"
    );
    assert_eq!(buffer.raw.iter().copied().collect::<Vec<_>>(), b"abc");
}

#[test]
fn terminal_query_and_hvp_scanners_survive_split_output_chunks() {
    let mut buffer = make_buf(4, 40, &[], &[], 0);

    assert!(buffer.ingest(b"\x1b[").is_empty());
    assert!(buffer.ingest(b"6").is_empty());
    assert_eq!(buffer.ingest(b"n"), b"\x1b[1;1R");

    assert!(buffer.ingest(b"\x1b[2;").is_empty());
    assert!(buffer.ingest(b"3fX").is_empty());
    assert_eq!(buffer.parser.screen().cursor_position(), (1, 3));
}

#[test]
fn csi_3j_clears_meatshell_scrollback_even_when_split() {
    let mut buffer = make_buf(3, 20, &["old one", "old two"], &["current"], 2);
    buffer.raw.extend(b"old one\nold two\n");
    buffer.prev.push(hist_line("old two"));
    buffer.sel_anchor = Some((0, 0));
    buffer.sel_focus = Some((1, 2));

    let _ = buffer.ingest(b"\x1b[3");
    assert_eq!(buffer.history.len(), 2);
    let _ = buffer.ingest(b"J");

    assert!(buffer.history.is_empty());
    assert_eq!(buffer.view_offset, 0);
    assert!(buffer.raw.is_empty());
    assert!(buffer.sel_anchor.is_none());
    assert!(buffer.sel_focus.is_none());
}

#[test]
fn unchanged_rows_reuse_the_render_cache_until_they_change() {
    let mut buffer = make_buf(4, 40, &[], &[], 0);
    buffer.ingest(b"alpha\r\nbeta");
    buffer.render();

    // A full render caches every row and clears the dirty map.
    assert!(buffer.live_cache.iter().all(Option::is_some));
    assert!(buffer.dirty.as_ref().unwrap().iter().all(|d| !d));

    // No new output → the second render reuses every cached row unchanged.
    buffer.render();
    assert!(buffer.dirty.as_ref().unwrap().iter().all(|d| !d));

    // Rewriting one line dirties only that row; the rest stay cached.
    buffer.ingest(b"\x1b[H\x1b[2Kalpha edited");
    let dirty = buffer.dirty.as_ref().unwrap();
    assert_eq!(dirty.iter().filter(|d| **d).count(), 1);
    buffer.render();
    assert_eq!(buffer.displayed_text[0], "alpha edited");
    assert_eq!(buffer.displayed_text[1], "beta");
    assert!(buffer.dirty.as_ref().unwrap().iter().all(|d| !d));
}

#[test]
fn invalidate_render_cache_forces_a_full_rebuild() {
    let mut buffer = make_buf(4, 40, &[], &[], 0);
    buffer.ingest(b"hello");
    buffer.render();
    assert!(buffer.dirty.as_ref().unwrap().iter().all(|d| !d));

    buffer.invalidate_render_cache();
    assert!(buffer.dirty.is_none());

    // The next render rebuilds everything and re-arms the dirty map.
    buffer.render();
    assert!(buffer.dirty.as_ref().unwrap().iter().all(|d| !d));
    assert!(buffer.live_cache.iter().all(Option::is_some));
}

#[test]
fn incoming_output_keeps_a_scrolled_view_anchored() {
    let mut buffer = make_buf(3, 20, &[], &[], 0);
    let _ = buffer.ingest(b"one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix");
    assert!(!buffer.history.is_empty());

    buffer.view_offset = 1;
    buffer.render();
    let before = buffer.displayed_text.clone();
    let old_offset = buffer.view_offset;

    let _ = buffer.ingest(b"\r\nseven");
    buffer.render();

    assert_eq!(buffer.view_offset, old_offset + 1);
    assert_eq!(buffer.displayed_text, before);
}

#[test]
fn long_unbroken_output_is_captured_before_it_wraps_off_screen() {
    let mut buffer = make_buf(3, 10, &[], &[], 0);
    let output = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let _ = buffer.ingest(output);
    buffer.render();

    assert!(
        !buffer.history.is_empty(),
        "wrapped rows must enter scrollback even without newline bytes"
    );
    let rendered = buffer
        .history
        .iter()
        .map(|line| line.0.as_str())
        .chain(buffer.displayed_text.iter().map(String::as_str))
        .collect::<String>();
    assert_eq!(rendered, String::from_utf8_lossy(output));
}
