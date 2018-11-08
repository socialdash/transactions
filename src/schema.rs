table! {
    accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        currency -> Varchar,
        address -> Varchar,
        name -> Nullable<Varchar>,
        kind -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    blockchain_transactions (hash) {
        hash -> Varchar,
        block_number -> Int8,
        currency -> Varchar,
        fee -> Numeric,
        confirmations -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        from_ -> Jsonb,
        to_ -> Jsonb,
    }
}

table! {
    pending_blockchain_transactions (hash) {
        hash -> Varchar,
        from_ -> Varchar,
        to_ -> Varchar,
        currency -> Varchar,
        value -> Numeric,
        fee -> Numeric,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    seen_hashes (hash, currency) {
        hash -> Varchar,
        block_number -> Int8,
        currency -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    strange_blockchain_transactions (hash) {
        hash -> Varchar,
        from_ -> Jsonb,
        to_ -> Jsonb,
        block_number -> Int8,
        currency -> Varchar,
        fee -> Numeric,
        confirmations -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        commentary -> Varchar,
    }
}

table! {
    transactions (id) {
        id -> Uuid,
        dr_account_id -> Uuid,
        cr_account_id -> Uuid,
        currency -> Varchar,
        value -> Numeric,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        gid -> Uuid,
    }
}

table! {
    tx_groups (id) {
        id -> Uuid,
        kind -> Varchar,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_id -> Uuid,
        blockchain_tx_id -> Nullable<Varchar>,
        base_tx -> Nullable<Uuid>,
        from_tx -> Nullable<Uuid>,
        to_tx -> Nullable<Uuid>,
        fee_tx -> Nullable<Uuid>,
        withdrawal_txs -> Jsonb,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        authentication_token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(accounts -> users (user_id));
joinable!(tx_groups -> users (user_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    blockchain_transactions,
    pending_blockchain_transactions,
    seen_hashes,
    strange_blockchain_transactions,
    transactions,
    tx_groups,
    users,
);
