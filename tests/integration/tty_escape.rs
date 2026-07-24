//! CSI final-byte classification for escape decoding.

/// Mirror of the TTY escape classifier: CSI finals are `@`…`~` (incl. `}`).
fn is_csi_final(b: u8) -> bool {
    (0x40..=0x7e).contains(&b)
}

#[test]
fn csi_final_includes_brace_and_tilde() {
    assert!(is_csi_final(b'}'));
    assert!(is_csi_final(b'~'));
    assert!(is_csi_final(b'A'));
    assert!(is_csi_final(b'm'));
    // Parameter / intermediate bytes — not treated as CSI terminator in the decoder
    // until index ≥ 2, but `[` itself is still in the ECMA-48 final range.
    assert!(!is_csi_final(b'0'));
    assert!(!is_csi_final(b';'));
    assert!(!is_csi_final(b' '));
}
