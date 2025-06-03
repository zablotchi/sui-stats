// @generated automatically by Diesel CLI.

diesel::table! {
    cp_sequence_numbers (cp_sequence_number) {
        cp_sequence_number -> Int8,
        tx_lo -> Int8,
        epoch -> Int8,
    }
}

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
        unique_object_ids -> Int8,
        unique_event_ids -> Int8,
    }
}

diesel::table! {
    watermarks (pipeline) {
        pipeline -> Text,
        epoch_hi_inclusive -> Int8,
        checkpoint_hi_inclusive -> Int8,
        tx_hi -> Int8,
        timestamp_ms_hi_inclusive -> Int8,
        reader_lo -> Int8,
        pruner_timestamp -> Timestamp,
        pruner_hi -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cp_sequence_numbers,
    sizes,
    watermarks,
);
