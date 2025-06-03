// @generated automatically by Diesel CLI.

diesel::table! {
    sizes (cp_sequence_number) {
        cp_sequence_number -> Int8,
        cp_summary_bytes -> Int8,
        cp_signatures_bytes -> Int8,
        cp_contents_bytes -> Int8,
        tx_count -> Int8,
        tx_bytes -> Int8,
        fx_bytes -> Int8,
        ev_bytes -> Int8,
        obj_count -> Int8,
        obj_bytes -> Int8,
    }
}
