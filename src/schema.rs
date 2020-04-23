table! {
    files (id) {
        id -> Integer,
        file_path -> Text,
        file_hash -> Binary,
        uuid -> Text,
        save_id -> Integer,
        created_at -> Timestamp,
        modified_at -> Timestamp,
    }
}

table! {
    saves (id) {
        id -> Integer,
        friendly_name -> Text,
        save_path -> Text,
        backup_path -> Text,
        user_id -> Integer,
        created_at -> Timestamp,
        modified_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        created_at -> Timestamp,
        modified_at -> Timestamp,
    }
}

joinable!(files -> saves (save_id));
joinable!(saves -> users (user_id));

allow_tables_to_appear_in_same_query!(
    files,
    saves,
    users,
);
