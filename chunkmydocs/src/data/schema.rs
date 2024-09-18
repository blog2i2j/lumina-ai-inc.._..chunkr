// @generated automatically by Diesel CLI.

diesel::table! {
    api_keys (key) {
        key -> Text,
        user_id -> Nullable<Text>,
        dataset_id -> Nullable<Text>,
        org_id -> Nullable<Text>,
        access_level -> Nullable<Text>,
        active -> Nullable<Bool>,
        deleted -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        deleted_by -> Nullable<Text>,
    }
}

diesel::table! {
    discounts (user_id) {
        user_id -> Text,
        usage_type -> Text,
        amount -> Nullable<Float8>,
    }
}

diesel::table! {
    invoices (invoice_id) {
        invoice_id -> Text,
        user_id -> Text,
        tasks -> Array<Nullable<Text>>,
        date_created -> Timestamp,
        date_paid -> Nullable<Timestamp>,
        invoice_status -> Text,
        amount_due -> Float8,
        total_pages -> Int4,
    }
}

diesel::table! {
    task_invoices (task_id) {
        task_id -> Text,
        invoice_id -> Text,
        usage_type -> Text,
        pages -> Int4,
        cost -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tasks (task_id) {
        task_id -> Text,
        user_id -> Nullable<Text>,
        api_key -> Nullable<Text>,
        file_name -> Nullable<Text>,
        file_size -> Nullable<Int8>,
        page_count -> Nullable<Int4>,
        segment_count -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
        status -> Nullable<Text>,
        task_url -> Nullable<Text>,
        input_location -> Nullable<Text>,
        output_location -> Nullable<Text>,
        configuration -> Nullable<Text>,
        message -> Nullable<Text>,
    }
}

diesel::table! {
    usage (id) {
        id -> Int4,
        user_id -> Nullable<Text>,
        usage -> Nullable<Int4>,
        usage_limit -> Nullable<Int4>,
        usage_type -> Nullable<Text>,
        unit -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    usage_type (id) {
        id -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        description -> Text,
        unit -> Nullable<Text>,
        cost_per_unit_dollars -> Nullable<Float8>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Text,
        customer_id -> Nullable<Text>,
        email -> Nullable<Text>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        tier -> Nullable<Text>,
        invoice_status -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    api_keys,
    discounts,
    invoices,
    task_invoices,
    tasks,
    usage,
    usage_type,
    users,
);
