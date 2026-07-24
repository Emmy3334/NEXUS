//! Classify finished escape sequences (ECMA-48 CSI / SS3 / Meta).

pub(super) fn is_finished(seq: &[u8]) -> bool {
    is_meta_chord(seq) || is_ss3(seq) || is_complete_csi(seq)
}

/// ECMA-48 CSI final byte (`@`…`~`), including `}`.
pub(super) fn is_csi_final(b: u8) -> bool {
    (0x40..=0x7e).contains(&b)
}

fn is_meta_chord(seq: &[u8]) -> bool {
    matches!(seq, [0x1b, b] if *b != b'[' && *b != b'O')
}

fn is_ss3(seq: &[u8]) -> bool {
    matches!(seq, [0x1b, b'O', _])
}

fn is_complete_csi(seq: &[u8]) -> bool {
    seq.len() >= 3
        && seq[0] == 0x1b
        && seq[1] == b'['
        && seq.last().is_some_and(|&b| is_csi_final(b))
}
