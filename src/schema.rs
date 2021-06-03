table! {
    stock (id) {
        id -> Int4,
        foreign_id -> Int4,
        availability -> Varchar,
        type_threshold -> Varchar,
        stocks_id -> Nullable<Int4>,
    }
}

table! {
    stocks (id) {
        id -> Int4,
        product_id -> Int4,
    }
}

joinable!(stock -> stocks (stocks_id));

allow_tables_to_appear_in_same_query!(
    stock,
    stocks,
);
